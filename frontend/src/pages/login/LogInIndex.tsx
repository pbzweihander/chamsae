export function LogInIndexPage() {
  const notes = new Array(40).fill("foobar"); // TODO: stub

  return (
    <div className="relative flex h-full w-full">
      <div className="h-full w-full overflow-y-scroll">
        {notes.map((note, i) => (
          <div className="chat chat-start">
            <div className="chat-bubble">
              {note} {i}
            </div>
          </div>
        ))}
      </div>
      <form className="chat chat-end absolute bottom-4 right-4">
        <div className="chat-bubble chat-bubble-primary">
          <textarea
            className="textarea w-full"
            placeholder="Jot something..."
          />
        </div>
        <div className="chat-footer">
          <input
            type="submit"
            className="btn btn-primary btn-sm mt-2"
            value="Chirp!"
          />
        </div>
      </form>
    </div>
  );
}
