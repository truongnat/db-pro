interface ErrorCellProps {
  message: string;
}

export function ErrorCell({ message }: ErrorCellProps) {
  return (
    <div
      className="rounded bg-destructive px-2 py-0.5 text-xs text-white"
      title={message}
    >
      {message}
    </div>
  );
}
