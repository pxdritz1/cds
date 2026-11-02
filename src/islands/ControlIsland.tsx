import Astal from "gi://Astal?version=4.0"
import Gtk from "gi://Gtk?version=4.0"
import app from "ags/gtk4/app"
import { execAsync } from "ags/process"

const { TOP, LEFT } = Astal.WindowAnchor

function run(command: string) {
  execAsync(command).catch((error) => console.error("[cds] command failed:", error))
}

function Action({ icon, label, command }: { icon: string; label: string; command: string }) {
  return (
    <button class="m3-action" onClicked={() => run(command)}>
      <box spacing={10}>
        <label class="symbol" label={icon} />
        <label label={label} />
      </box>
    </button>
  )
}

export function ControlIsland() {
  return (
    <window
      visible
      namespace="chloe-shell-control"
      name="cds-control-island"
      application={app}
      anchor={TOP | LEFT}
      marginTop={12}
      marginLeft={16}
      layer={Astal.Layer.TOP}
    >
      <box class="island control-island" spacing={10}>
        <menubutton class="island-trigger" tooltipText="Controles rápidos">
          <box spacing={9}>
            <label class="symbol" label="tune" />
            <label class="island-label" label="controles" />
            <label class="symbol muted-symbol" label="expand_more" />
          </box>
          <popover>
            <box class="popover-content" orientation={Gtk.Orientation.VERTICAL} spacing={8}>
              <label class="section-title" label="controles rápidos" xalign={0} />
              <Action icon="volume_up" label="alternar áudio" command="wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle" />
              <Action icon="brightness_6" label="aumentar brilho" command="brightnessctl set +5%" />
              <Action icon="brightness_low" label="diminuir brilho" command="brightnessctl set 5%-" />
              <Action icon="wifi" label="configurar rede" command="nm-connection-editor" />
              <Action icon="bluetooth" label="configurar bluetooth" command="blueman-manager" />
              <Gtk.Separator />
              <label class="helper" label="requer wpctl, brightnessctl, NetworkManager e blueman" wrap xalign={0} />
            </box>
          </popover>
        </menubutton>
        <box class="status-dot" />
      </box>
    </window>
  )
}
