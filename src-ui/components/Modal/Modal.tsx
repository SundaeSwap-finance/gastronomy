import { FC, ReactNode } from "react";

interface ModalProps {
  children: ReactNode;
  isOpen: boolean;
}

const Modal: FC<ModalProps> = ({ isOpen, children }) => {
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 flex items-center justify-center z-50 p-6">
      <div className="fixed inset-0 bg-black opacity-40" />
      <div className="bg-slate-950 rounded z-10 border border-lime-600 relative w-[50rem] max-w-full">
        {children}
      </div>
    </div>
  );
};

export default Modal;
