import { open } from "@tauri-apps/plugin-dialog";
import { Dispatch, FC, SetStateAction } from "react";

interface IFilePickerProps {
  fileName?: string;
  setFile: Dispatch<SetStateAction<string>>;
}

const FilePicker: FC<IFilePickerProps> = ({ setFile, fileName }) => {
  const handleFilePick = async () => {
    const selectedFile = await open({
      multiple: false,
      filters: [
        { name: "Executable UPLC", extensions: ["uplc", "flat", "tx", "json"] },
        { name: "UPLC Files", extensions: ["uplc"] },
        { name: "Flat Files", extensions: ["flat"] },
        { name: "Transaction Files", extensions: ["tx"] },
        { name: "JSON Files", extensions: ["json"] },
      ],
    });

    if (selectedFile) setFile(selectedFile as string);
  };

  return (
    <button
      className="border border-dashed border-lime-600 px-4 py-4 hover:bg-lime-600/15 transition-colors duration-300 ease-in-out"
      onClick={handleFilePick}
    >
      {fileName || "Click to pick a file"}
    </button>
  );
};

export default FilePicker;
