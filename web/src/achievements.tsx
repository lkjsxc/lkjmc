import { useState } from "react";
import { money, type Data } from "./api";
import { renderSystemMessage, t } from "./i18n";
import { Card, Empty } from "./ui";

export function Achievements({ groups }: { groups: Data[] }) {
  const [ownerId, setOwnerId] = useState("");
  const selected =
    groups.find((group) => group.owner.id === ownerId) ?? groups[0];
  return (
    <Card title={t("text.achievements")}>
      {selected ? (
        <>
          <label className="field">
            <span>{t("text.achievement_progress_for")}</span>
            <select
              value={selected.owner.id}
              onChange={(event) => setOwnerId(event.target.value)}
            >
              {groups.map(({ owner }) => (
                <option key={owner.id} value={owner.id}>
                  {owner.kind === "account" ? t("text.personal") : owner.name}
                </option>
              ))}
            </select>
          </label>
          <div className="grid three">
            {selected.achievements.map((achievement: Data) => (
              <div
                className={`achievement ${achievement.earned_at ? "earned" : ""}`}
                key={achievement.key}
              >
                <span className="eyebrow">
                  {selected.owner.kind === "team"
                    ? selected.owner.name
                    : t("text.personal")}
                </span>
                <h3>
                  {achievement.title_message
                    ? renderSystemMessage(achievement.title_message)
                    : achievement.title}
                </h3>
                <p>
                  {achievement.description_message
                    ? renderSystemMessage(achievement.description_message)
                    : achievement.description}
                </p>
                <progress
                  value={achievement.progress}
                  max={achievement.target}
                  aria-label={
                    achievement.title_message
                      ? renderSystemMessage(achievement.title_message)
                      : achievement.title
                  }
                />
                <small>
                  {money(achievement.progress)} / {money(achievement.target)}{" "}
                  {achievement.earned_at ? t("text.earned") : ""}
                </small>
                <div className="rewards">
                  {achievement.land_chunks > 0 && (
                    <span>
                      {t("text.land")} {money(achievement.land_chunks)}
                    </span>
                  )}
                  {achievement.coins > 0 && (
                    <span>
                      {money(achievement.coins)} {t("text.coins")}
                    </span>
                  )}
                </div>
              </div>
            ))}
          </div>
        </>
      ) : (
        <Empty>{t("text.no_achievements_available")}</Empty>
      )}
    </Card>
  );
}
