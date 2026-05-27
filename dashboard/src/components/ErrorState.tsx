export function ErrorState({ message }: { message: string }) {
  return (
    <section className="error">
      <strong>Dashboard data unavailable</strong>
      <p>{message}</p>
    </section>
  );
}
