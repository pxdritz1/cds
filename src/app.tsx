import app from "ags/gtk4/app"
import style from "./theme.css"
import { ControlIsland } from "./islands/ControlIsland"
import { CoreIsland } from "./islands/CoreIsland"
import { ActivityIsland } from "./islands/ActivityIsland"

app.start({
  css: style,
  gtkTheme: "Adwaita",
  main() {
    return (
      <>
        <ControlIsland />
        <CoreIsland />
        <ActivityIsland />
      </>
    )
  },
})
