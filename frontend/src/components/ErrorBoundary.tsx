import { Component, type ErrorInfo, type ReactNode } from 'react';
import { useTranslation } from 'react-i18next';

interface Props {
  children: ReactNode;
  /** Optional label used in dev logs to identify which subtree crashed. */
  scope?: string;
  /** Fallback override. Defaults to the standard "Something went wrong" card. */
  fallback?: ReactNode;
}

interface State {
  error: Error | null;
}

/**
 * Catches render-phase errors in its children so a single broken widget
 * (e.g. an EventPicker crash from malformed data) doesn't blank the whole
 * page. React 19 still requires a class component for
 * `getDerivedStateFromError` — hooks can't hook into render errors.
 *
 * Logs to console.error so devs see the stack; renders a small fallback
 * card with a Reload button. Multiple instances are intentional: each
 * major widget (EventPicker, BetList, Leaderboard) wraps itself so a crash
 * in one doesn't take the others down.
 */
export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo): void {
    // eslint-disable-next-line no-console
    console.error(`[ErrorBoundary${this.props.scope ? `:${this.props.scope}` : ''}]`, error, info.componentStack);
  }

  private handleReload = () => {
    this.setState({ error: null });
  };

  render() {
    if (this.state.error) {
      return this.props.fallback ?? <DefaultFallback onReload={this.handleReload} />;
    }
    return this.props.children;
  }
}

function DefaultFallback({ onReload }: { onReload: () => void }) {
  const { t } = useTranslation();
  return (
    <div className="error-boundary" role="alert">
      <div className="error-boundary-icon" aria-hidden="true">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
          <path d="M12 9 L12 13" />
          <circle cx="12" cy="16.5" r="0.6" fill="currentColor" stroke="none" />
          <path d="M10.3 3.6 L3.6 17.4 Q 2.6 19.6 4.8 19.6 L 19.2 19.6 Q 21.4 19.6 20.4 17.4 L 13.7 3.6 Q 12.7 1.4 10.3 3.6 Z" />
        </svg>
      </div>
      <p className="error-boundary-title">{t('errors.boundaryTitle')}</p>
      <p className="error-boundary-body">{t('errors.boundaryBody')}</p>
      <button type="button" className="btn-error-reload" onClick={onReload}>
        {t('errors.boundaryReload')}
      </button>
    </div>
  );
}