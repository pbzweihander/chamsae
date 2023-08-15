export default function Post({
  params: { id },
}: {
  params: { id: string };
}) {
  return (
    <div className="border rounded p-4">
      {id}
    </div>
  );
}
