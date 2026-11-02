import Astal from "gi://Astal?version=4.0"
import Gtk from "gi://Gtk?version=4.0"
import app from "ags/gtk4/app"
import { execAsync } from "ags/process"

const { TOP, RIGHT } = Astal.WindowAnchor

function run(command: string) {
  execAsync(command).catch((error) => console.error("[cds] action failed:", error))
}

export function ActivityIsland() {
  return (
    <window
      visible
      namespace="chloe-shell-activity"
      name="cds-activity-island"
      application={app}
      anchor={TOP | RIGHT}
      marginTop={12}
      marginRight={16}
      layer={Astal.Layer.TOP}
    >
      <box class="island activity-island" spacing={8}>
        <menubutton class="island-trigger" tooltipText="Atividades e ferramentas">
          <box spacing={9}>
            <label class="symbol" label="widgets" />
            <label class="island-label" label="ferramentas" />
            <label class="symbol muted-symbol" label="expand_more" />
          </box>
          <popover>
            <box class="popover-content" orientation={Gtk.Orientation.VERTICAL} spacing={8}>
              <label class="section-title" label="espaço de trabalho" xalign={0} />
              <button class="m3-action" onClicked={() => run("fuzzel")}>
                <box spacing={10}>
                  <label class="symbol" label="apps" />
                  <label label="abrir aplicativos" />
                </box>
              </button>
              <button class="m3-action" onClicked={() => run("cliphist list | fuzzel --dmenu | cliphist decode | wl-copy")}>
                <box spacing={10}>
                  <label class="symbol" label="content_paste" />
                  <label label="histórico da área de transferência" />
                </box>
              </button>
              <button class="m3-action" onClicked={() => run("makoctl dismiss -a")}>
                <box spacing={10}>
                  <label class="symbol" label="notifications_off" />
                  <label label="limpar notificações" />
                </box>
              </button>
              <Gtk.Separator />
              <label class="helper" label="clipboard: cliphist + wl-clipboard · launcher: fuzzel" wrap xalign={0} />
            </box>
          </popover>
        </menubutton>
        <menubutton class="icon-button settings-button" tooltipText="preferências visuais">
          <label class="symbol" label="settings" />
          <popover>
            <box class="popover-content" orientation={Gtk.Orientation.VERTICAL} spacing={8}>
              <label class="section-title" label="chloe's design system" xalign={0} />
              <box class="preference-row">
                <label class="symbol" label="palette" />
                <box orientation={Gtk.Orientation.VERTICAL} spacing={2}>
                  <label label="Material 3 Expressive" xalign={0} />
                  <label class="helper" label="superfícies tonais e formas expressivas" xalign={0} />
                </box>
              </box>
              <box class="preference-row">
                <label class="symbol" label="text_fields" />
                <box orientation={Gtk.Orientation.VERTICAL} spacing={2}>
                  <label label="Google Sans Flex" xalign={0} />
                  <label class="helper" label="tipografia principal" xalign={0} />
                </box>
              </box>
              <box class="preference-row">
                <label class="symbol" label="shapes" />
                <box orientation={Gtk.Orientation.VERTICAL} spacing={2}>
                  <label label="Material Symbols Rounded" xalign={0} />
                  <label class="helper" label="ícones e indicadores" xalign={0} />
                </box>
              </box>
              <Gtk.Separator />
              <label class="helper" label="o editor de cores, temas dinâmicos e preferências persistentes será conectado nesta área." wrap xalign={0} />
            </box>
          </popover>
        </menubutton>
      </box>
    </window>
  )
}
