export interface HarbourAppBarProps {
  readonly homeUrl: string;
  readonly appName: string;
}

export function HarbourAppBar({ homeUrl, appName }: HarbourAppBarProps) {
  return (
    <header className="harbour-chrome">
      <div className="harbour-chrome__inner">
        <a href={homeUrl} className="harbour-chrome__brand">
          <span className="harbour-chrome__brand-mark" aria-hidden />
          Harbour
        </a>
        <span className="harbour-chrome__divider" aria-hidden />
        <span className="harbour-chrome__app">{appName}</span>
        <span className="harbour-chrome__spacer" />
        <a href={homeUrl} className="harbour-chrome__home-link">
          All apps
        </a>
      </div>
    </header>
  );
}
