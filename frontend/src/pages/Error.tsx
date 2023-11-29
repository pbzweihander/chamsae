interface Props {
  error: Error;
}

export default function ErrorPage(props: Props) {
  return (
    <div className="p-4">
      <div>Error!</div>
      <div>{props.error.message}</div>
    </div>
  );
}
