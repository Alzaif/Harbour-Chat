import type { VoteValue } from '../../api/types';

export interface VoteControlsProps {
  score: number;
  myVote: VoteValue;
  disabled?: boolean;
  onVote: (value: VoteValue) => void;
}

export function VoteControls({ score, myVote, disabled, onVote }: VoteControlsProps) {
  return (
    <div className="vote-controls" role="group" aria-label="Votes">
      <button
        type="button"
        className={`vote-controls__btn${myVote === 1 ? ' vote-controls__btn--active' : ''}`}
        aria-label="Upvote"
        aria-pressed={myVote === 1}
        disabled={disabled}
        onClick={() => onVote(1)}
      >
        ▲
      </button>
      <span className="vote-controls__score" aria-label={`Score ${score}`}>
        {score}
      </span>
      <button
        type="button"
        className={`vote-controls__btn${myVote === -1 ? ' vote-controls__btn--active' : ''}`}
        aria-label="Downvote"
        aria-pressed={myVote === -1}
        disabled={disabled}
        onClick={() => onVote(-1)}
      >
        ▼
      </button>
    </div>
  );
}
