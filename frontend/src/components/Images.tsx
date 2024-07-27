import { useState } from "react";

interface propsType {
  files: {
    url: string;
    mediaType: string;
    alt?: string;
  }[];
}

const Images = ({ files }: propsType) => {
  const [clicked, setClicked] = useState(false);
  const [url, setUrl] = useState<string>("");

  const handleImageClick = (url: string) => {
    setClicked(!clicked);
    setUrl(url);
  };

  return (
    <div>
      {files &&
        files.map((file) => (
          <img
            key={file.url}
            src={file.url}
            className="max-w-[24rem] cursor-pointer"
            onClick={() => handleImageClick(file.url)}
          />
        ))}
      {clicked && (
        <div
          className="fixed left-0 top-0 z-10 flex items-center justify-center bg-slate-500/40"
          onClick={() => setClicked(!clicked)}
        >
          <img className="h-screen w-screen object-scale-down" src={url} />
        </div>
      )}
    </div>
  );
};

export default Images;
