export function Spinner({ label }: { label?: string }) {
  return (
    <div className="spinner-container">
      <div className="spinner" />
      {label && <span className="spinner-label">{label}</span>}
    </div>
  );
}
