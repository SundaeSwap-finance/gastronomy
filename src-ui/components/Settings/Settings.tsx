import {
  ChangeEvent,
  FC,
  useCallback,
  useEffect,
  useState,
} from "react";
import { Store } from "@tauri-apps/plugin-store";
import { ISettings } from "../../types";
import objectPath from "object-path";
import Modal from "../Modal";

interface ISettingsProps {
  isOpen: boolean;
  onClose: () => void;
}

const Settings: FC<ISettingsProps> = ({ isOpen, onClose }) => {
  const [config, setConfig] = useState<ISettings | null>(null);

  const [store, setStore] = useState<Store | null>(null);
  useEffect(() => {
    (async () => {
      setStore(await Store.load("settings.json"));
    })();
  }, []);

  const onSettingChanged = useCallback(
    (event: ChangeEvent<HTMLInputElement>) => {
      setConfig((oldConfig) => {
        const newConfig = { ...oldConfig };
        const updated = event.target.name;
        const value = event.target.value;
        if (updated === "blockfrost.key") {
          if (value) {
            objectPath.set(newConfig, "blockfrost.key", value);
          } else {
            objectPath.del(newConfig, "blockfrost");
          }
        }
        return newConfig;
      });
    },
    [],
  );

  const saveSettings = useCallback(async () => {
    if (!store) {
      return;
    }
    try {
      await store.set("config", config);
      await store.save();
    } catch (e) {
      console.error(e);
    }
  }, [store, config]);

  useEffect(() => {
    if (!store) {
      return;
    }
    (async () => {
      const data = await store.get<ISettings>("config") ?? {};
      setConfig(data);
    })();
  }, [store]);

  const handleKeyPress = useCallback(
    (event: KeyboardEvent) => {
      if (event.key === "Enter" || event.key === "Escape") {
        onClose();
      }
    },
    [onClose],
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyPress);
    return () => {
      window.removeEventListener("keydown", handleKeyPress);
    };
  }, [handleKeyPress]);

  return (
    <Modal className="w-[32rem] px-2" isOpen={isOpen} onClose={onClose}>
      <h1 className="absolute left-4 -top-3 bg-slate-950 px-2">Settings</h1>
      <form className="px-4 pt-4 pb-4 flex flex-col overflow-auto">
        <div className="flex flex-col">
          <label htmlFor="blockfrost.key">Blockfrost API Key</label>
          <input
            id="blockfrost.key"
            name="blockfrost.key"
            type="text"
            className="my-2"
            placeholder="Blockfrost API Key"
            value={config?.blockfrost?.key || ""}
            onChange={onSettingChanged}
            onBlur={saveSettings}
          />
        </div>
        <button className="hover:underline" onClick={onClose}>
          Done
        </button>
      </form>
    </Modal>
  );
};

export default Settings;
