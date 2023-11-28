"use client";

import { useRouter } from "next/navigation";
import { useLayoutEffect, useState } from "react";
import { createPortal } from "react-dom";

interface Props {
  children?: React.ReactNode;
  onClose?(): void;
}

export default function Modal(props: Props) {
  const [modalContainer, setModalContainer] = useState<HTMLDivElement>();
  useLayoutEffect(() => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    setModalContainer(container);
    return () => {
      document.body.removeChild(container);
      setModalContainer(undefined);
    };
  }, []);

  return modalContainer && createPortal(<ModalBody {...props} />, modalContainer);
}

function ModalBody(props: Props) {
  const router = useRouter();

  function handleClick(e: React.MouseEvent<HTMLDivElement>) {
    if (e.currentTarget === e.target) {
      if (props.onClose) {
        props.onClose();
      } else {
        router.back();
      }
    }
  }

  return (
    <div className="fixed inset-0 bg-black/[.05] px-16 py-24" onClick={handleClick}>
      <div className="w-full max-w-screen-md mx-auto rounded bg-white drop-shadow-lg p-8">
        {props.children}
      </div>
    </div>
  );
}
