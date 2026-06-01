import { useCallback, useMemo, useRef, useState } from 'react';
import { Device } from 'mediasoup-client';
import { api } from '../../api/client';

type Transport = any;
type Consumer = any;

type VoiceConnectionState = 'idle' | 'connecting' | 'connected' | 'reconnecting' | 'failed';

export interface UseVoiceMediaClientResult {
  connectionState: VoiceConnectionState;
  localStream: MediaStream | null;
  remoteStreams: { producerId: string; userId: string; stream: MediaStream }[];
  error: string | null;
  start: (channelId: string) => Promise<void>;
  stop: () => Promise<void>;
}

export function useVoiceMediaClient(): UseVoiceMediaClientResult {
  const [connectionState, setConnectionState] = useState<VoiceConnectionState>('idle');
  const [localStream, setLocalStream] = useState<MediaStream | null>(null);
  const [remoteStreams, setRemoteStreams] = useState<
    { producerId: string; userId: string; stream: MediaStream }[]
  >([]);
  const [error, setError] = useState<string | null>(null);
  const activeSessionRef = useRef<{
    channelId: string;
    sessionId: string;
    sendTransport: Transport;
    recvTransport: Transport;
    remoteUsersByProducer: Map<string, string>;
    pollId: number | null;
    localTrack: MediaStreamTrack;
  } | null>(null);
  const consumersRef = useRef<Map<string, Consumer>>(new Map());

  const stop = useCallback(async () => {
    const active = activeSessionRef.current;
    if (active?.pollId) window.clearInterval(active.pollId);
    for (const consumer of consumersRef.current.values()) {
      consumer.close();
    }
    consumersRef.current.clear();
    if (active) {
      active.sendTransport.close();
      active.recvTransport.close();
      active.localTrack.stop();
      void api.closeVoiceSession(active.channelId, active.sessionId).catch(() => {});
    }
    if (localStream) {
      for (const track of localStream.getTracks()) track.stop();
    }
    setLocalStream(null);
    setRemoteStreams([]);
    activeSessionRef.current = null;
    setConnectionState('idle');
  }, [localStream]);

  const start = useCallback(async (channelId: string) => {
    try {
      await stop();
      setError(null);
      setConnectionState('connecting');
      const stream = await navigator.mediaDevices.getUserMedia({
        audio: {
          echoCancellation: true,
          noiseSuppression: true,
          autoGainControl: true,
        },
        video: false,
      });
      setLocalStream(stream);

      const requestId = crypto.randomUUID();
      const sessionResp = await api.bootstrapVoiceSession(channelId, requestId);
      if (!sessionResp.ok) {
        throw new Error('voice session bootstrap failed');
      }
      const session = sessionResp.payload;
      const audioTrack = stream.getAudioTracks()[0];
      if (!audioTrack) throw new Error('No audio track available');

      const sendTransportResp = await api.createVoiceTransport(
        channelId,
        crypto.randomUUID(),
        session.session_id,
        'send',
      );
      if (!sendTransportResp.ok) throw new Error('create send transport failed');

      const recvTransportResp = await api.createVoiceTransport(
        channelId,
        crypto.randomUUID(),
        session.session_id,
        'recv',
      );
      if (!recvTransportResp.ok) throw new Error('create recv transport failed');

      const device = new Device();
      await device.load({
        routerRtpCapabilities: session.router_rtp_capabilities as any,
      });

      const sendTransport = device.createSendTransport({
        id: sendTransportResp.payload.transport_id,
        iceParameters: sendTransportResp.payload.ice_parameters as any,
        iceCandidates: sendTransportResp.payload.ice_candidates as any,
        dtlsParameters: sendTransportResp.payload.dtls_parameters as any,
      });

      sendTransport.on('connect', ({ dtlsParameters }, callback, errback) => {
        void api
          .connectVoiceTransport(
            channelId,
            sendTransport.id,
            crypto.randomUUID(),
            session.session_id,
            dtlsParameters as any,
          )
          .then(() => callback())
          .catch((err) => errback(err as Error));
      });

      sendTransport.on('produce', ({ kind, rtpParameters }, callback, errback) => {
        void api
          .createVoiceProducer(
            channelId,
            crypto.randomUUID(),
            session.session_id,
            sendTransport.id,
            kind as 'audio',
            rtpParameters as any,
          )
          .then((resp) => {
            if (!resp.ok) throw new Error('producer create rejected');
            callback({ id: resp.payload.producer_id });
          })
          .catch((err) => errback(err as Error));
      });

      sendTransport.on('connectionstatechange', (state) => {
        if (state === 'connected') setConnectionState('connected');
        else if (state === 'connecting') setConnectionState('connecting');
        else if (state === 'disconnected') setConnectionState('reconnecting');
        else if (state === 'failed') setConnectionState('failed');
      });

      await sendTransport.produce({ track: audioTrack });

      const recvTransport = device.createRecvTransport({
        id: recvTransportResp.payload.transport_id,
        iceParameters: recvTransportResp.payload.ice_parameters as any,
        iceCandidates: recvTransportResp.payload.ice_candidates as any,
        dtlsParameters: recvTransportResp.payload.dtls_parameters as any,
      });

      recvTransport.on('connect', ({ dtlsParameters }, callback, errback) => {
        void api
          .connectVoiceTransport(
            channelId,
            recvTransport.id,
            crypto.randomUUID(),
            session.session_id,
            dtlsParameters as any,
          )
          .then(() => callback())
          .catch((err) => errback(err as Error));
      });

      const remoteUsersByProducer = new Map<string, string>();
      const poll = window.setInterval(() => {
        void (async () => {
          const producers = await api.listRemoteVoiceProducers(channelId, session.session_id);
          for (const producer of producers.producers) {
            if (consumersRef.current.has(producer.producer_id)) continue;
            const resp = await api.createVoiceConsumer(
              channelId,
              crypto.randomUUID(),
              session.session_id,
              recvTransport.id,
              producer.producer_id,
              device.rtpCapabilities as unknown,
            );
            if (!resp.ok) continue;
            remoteUsersByProducer.set(producer.producer_id, producer.user_id);
            const consumer = await recvTransport.consume({
              id: resp.payload.consumer_id,
              producerId: resp.payload.producer_id,
              kind: resp.payload.kind as 'audio',
              rtpParameters: resp.payload.rtp_parameters as any,
            });
            consumersRef.current.set(producer.producer_id, consumer);
            const remote = new MediaStream([consumer.track]);
            setRemoteStreams((prev) => [
              ...prev.filter((p) => p.producerId !== producer.producer_id),
              { producerId: producer.producer_id, userId: producer.user_id, stream: remote },
            ]);
          }
        })().catch(() => {});
      }, 1500);

      activeSessionRef.current = {
        channelId,
        sessionId: session.session_id,
        sendTransport,
        recvTransport,
        remoteUsersByProducer,
        pollId: poll,
        localTrack: audioTrack,
      };

      setConnectionState('connected');
    } catch (e) {
      setConnectionState('failed');
      setError(e instanceof Error ? e.message : 'Voice media setup failed');
      await stop();
    }
  }, [stop]);

  return useMemo(
    () => ({ connectionState, localStream, remoteStreams, error, start, stop }),
    [connectionState, localStream, remoteStreams, error, start, stop],
  );
}
