import { useCallback, useEffect, useId, useState } from "react";
import { fmt, t } from "../i18n";
import { api } from "../lib/ipc";

type Operation = "save" | "check";

function operationError(error: unknown): string {
  if (error instanceof Error && error.message) return error.message;
  if (typeof error === "string" && error) return error;
  return "Could not check LanChat.";
}

export function LanChatShareRow() {
  const inputId = useId();
  const helpId = useId();
  const [origin, setOrigin] = useState("");
  const [loading, setLoading] = useState(true);
  const [operation, setOperation] = useState<Operation | null>(null);
  const [message, setMessage] = useState<{ error: boolean; text: string } | null>(null);

  useEffect(() => {
    let active = true;
    void api.getLanChatShareSettings()
      .then((settings) => {
        if (active) setOrigin(settings.origin ?? "");
      })
      .catch((error: unknown) => {
        if (active) setMessage({ error: true, text: operationError(error) });
      })
      .finally(() => {
        if (active) setLoading(false);
      });
    return () => {
      active = false;
    };
  }, []);

  const save = useCallback(async (checkAfterSave: boolean) => {
    if (loading || operation !== null) return;
    setOperation(checkAfterSave ? "check" : "save");
    setMessage(null);
    try {
      const saved = await api.setLanChatShareOrigin(origin.trim() || null);
      setOrigin(saved.origin ?? "");
      if (!checkAfterSave) {
        setMessage({ error: false, text: t("Saved") });
        return;
      }
      const connection = await api.checkLanChatShare();
      setMessage({
        error: false,
        text: fmt("Connected to %@", connection.roomName),
      });
    } catch (error) {
      setMessage({ error: true, text: operationError(error) });
    } finally {
      setOperation(null);
    }
  }, [loading, operation, origin]);

  return (
    <div className="kiri-settings-card kiri-lanchat-row">
      <div className="kiri-lanchat-copy">
        <strong>{t("LanChat sharing")}</strong>
        <span id={helpId}>
          {t("Screenshots are sent only when you choose Send to LanChat.")}
        </span>
        <label className="kiri-lanchat-field" htmlFor={inputId}>
          <span>{t("LanChat address")}</span>
          <input
            id={inputId}
            type="text"
            inputMode="url"
            spellCheck={false}
            autoComplete="off"
            value={origin}
            placeholder="127.0.0.1:8787"
            aria-describedby={helpId}
            disabled={loading || operation !== null}
            onChange={(event) => {
              setOrigin(event.target.value);
              setMessage(null);
            }}
            onKeyDown={(event) => {
              if (event.key !== "Enter") return;
              event.preventDefault();
              void save(true);
            }}
          />
        </label>
        <span className="kiri-lanchat-hint">
          {t("Optional. Leave blank to discover a local room automatically.")}
        </span>
        {message && (
          <span
            className={message.error ? "kiri-lanchat-status is-error" : "kiri-lanchat-status"}
            role={message.error ? "alert" : "status"}
          >
            {t(message.text)}
          </span>
        )}
      </div>
      <div className="kiri-lanchat-actions">
        <button
          type="button"
          className="kiri-button kiri-button--secondary"
          disabled={loading || operation !== null}
          onClick={() => void save(false)}
        >
          {t("Save")}
        </button>
        <button
          type="button"
          className="kiri-button kiri-button--secondary"
          disabled={loading || operation !== null}
          onClick={() => void save(true)}
        >
          {t(operation === "check" ? "Checking…" : "Check")}
        </button>
      </div>
    </div>
  );
}
