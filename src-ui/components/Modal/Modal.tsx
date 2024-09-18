import { FC, ReactNode } from "react";
import cx from "classnames";

interface IModalProps {
  className?: string;
  children: ReactNode;
  isOpen: boolean;
  onClose?: () => void;
}

const Modal: FC<IModalProps> = ({ className, isOpen, onClose, children }) => {
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 flex items-center justify-center z-50 p-6">
      <div className="fixed inset-0 bg-black opacity-40" onClick={onClose} />
      <div
        className={cx(
          "bg-slate-950 rounded z-10 border border-lime-600 relative max-w-full",
          className,
        )}
      >
        {children}
      </div>
    </div>
  );
};

export default Modal;
