import Astal from "gi://Astal?version=4.0"
import Gtk from "gi://Gtk?version=4.0"
import GLib from "gi://GLib"
import app from "ags/gtk4/app"
import { createPoll } from "ags/time"
import { execAsync } from "ags/process"

const { TOP } = Astal.WindowAnchor

function run(command: string) {
  execAsync(command).catch((error) => console.error("[cds] media command failed:", error))
}

export function CoreIsland() {
  const clock = createPoll("", 1000, () =>
    GLib.DateTime.new_now_local().format("%H:%M")!,
  )
  const date = createPoll("", 60_000, () =>
    GLib.DateTime.new_now_local().format("%A, %d %B")!,
  )

  return (
    <window
      visible
      namespace="chloe-shell-core"
      name="cds-core-island"
      application={app}
      anchor={TOP}
      marginTop={12}
      layer={Astal.Layer.TOP}
    >
      <box class="island core-island" spacing={14}>
        <box orientation={Gtk.Orientation.VERTICAL} spacing={1}>
          <label class="clock" label={clock} />
          <label class="date-label" label={date} />
        </box>
        <box class="island-divider" />
        <box class="media-controls" spacing={4}>
          <button class="icon-button" tooltipText="Anterior" onClicked={() => run("playerctl previous")}>
            <label class="symbol" label="skip_previous" />
          </button>
          <button class="icon-button play-button" tooltipText="Reproduzir / pausar" onClicked={() => run("playerctl play-pause")}>
            <label class="symbol" label="play_pause" />
          </button>
          <button class="icon-button" tooltipText="Próxima" onClicked={() => run("playerctl next")}>
            <label class="symbol" label="skip_next" />
          </button>
        </box>
      </box>
    </window>
  )
}
