import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { ArrowLeft } from "lucide-react";
import { cn } from "@/lib/utils";
import SettingsSidebar from "../components/settings/SettingsSidebar";
import SettingsContent from "../components/settings/SettingsContent";

export default function Settings() {
  const navigate = useNavigate();
  const [selectedItem, setSelectedItem] = useState<string>("General");

  const handleSelect = (item: string) => {
    setSelectedItem(item);
  };

  return (
    <div className="flex flex-col h-screen">
      <div
        className={cn(
          "flex flex-row items-center flex-none min-h-[35px]",
          "bg-zinc-800/60 px-4",
        )}
      >
        <button
          onClick={() => navigate("/")}
          className={cn(
            "flex items-center justify-center",
            "h-6 w-6 rounded-md",
            "text-zinc-400 hover:text-zinc-100",
            "hover:bg-zinc-700/60",
            "transition-colors duration-150",
          )}
          aria-label="Back to search"
        >
          <ArrowLeft className="h-4 w-4" />
        </button>
      </div>

      <div
        className={cn(
          "flex flex-1 min-h-0 flex-row items-stretch ",
          "bg-zinc-800/60",
        )}
      >
        <SettingsSidebar selectedItem={selectedItem} onSelect={handleSelect} />
        <SettingsContent item={selectedItem} />
      </div>
    </div>
  );
}
