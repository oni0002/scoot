import { Component, ErrorInfo, ReactNode } from 'react';

interface Props {
    children: ReactNode;
}

interface State {
    hasError: boolean;
    error?: Error;
}

export class ErrorBoundary extends Component<Props, State> {
    constructor(props: Props) {
        super(props);
        this.state = { hasError: false };
    }

    static getDerivedStateFromError(error: Error): State {
        return { hasError: true, error };
    }

    componentDidCatch(error: Error, errorInfo: ErrorInfo) {
        console.error('ErrorBoundary caught an error:', error, errorInfo);
    }

    render() {
        if (this.state.hasError) {
            return (
                <div
                    role="alert"
                    className="h-screen w-screen flex items-center justify-center bg-base-100 text-base-content p-6"
                >
                    <div className="w-full max-w-md space-y-3">
                        <h2 className="text-lg font-medium">Something went wrong</h2>
                        <p className="text-sm text-muted">
                            Scoot hit an unexpected error. Try again, and restart Scoot if it keeps
                            happening.
                        </p>
                        {this.state.error && (
                            <details className="text-xs">
                                <summary className="cursor-pointer text-muted">
                                    Error details
                                </summary>
                                <pre className="mt-2 whitespace-pre-wrap break-words rounded-sm bg-base-200 p-2">
                                    {this.state.error.toString()}
                                </pre>
                            </details>
                        )}
                        <button
                            className="btn btn-sm btn-primary"
                            onClick={() => {
                                this.setState({ hasError: false, error: undefined });
                            }}
                        >
                            Try again
                        </button>
                    </div>
                </div>
            );
        }

        return this.props.children;
    }
}
