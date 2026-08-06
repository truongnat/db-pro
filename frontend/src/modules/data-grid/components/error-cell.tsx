interface ErrorCellProps {
  message: string;
}

export function ErrorCell({ message }: ErrorCellProps) {
  return (
    <div
      className="rounded px-2 py-0.5 text-xs"
      style={{ backgroundColor: "var(--color-error, #ef4444)", color: "#fff" }}
      title={message}
    >
      {message}
    </div>
  );
}
