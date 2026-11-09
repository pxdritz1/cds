-- Chloe Dynamic Shell — high-contrast Matugen colorscheme
vim.cmd("highlight clear")
vim.o.background = "{{ mode }}"
vim.g.colors_name = "chloe-dynamic-shell"

local hl = vim.api.nvim_set_hl
local c = {
  bg = "{{ colors.background.default.hex }}",
  fg = "{{ colors.on_background.default.hex }}",
  surface = "{{ colors.surface.default.hex }}",
  surface_low = "{{ colors.surface_container_low.default.hex }}",
  surface_container = "{{ colors.surface_container.default.hex }}",
  surface_variant = "{{ colors.surface_variant.default.hex }}",
  primary = "{{ colors.primary.default.hex }}",
  on_primary = "{{ colors.on_primary.default.hex }}",
  primary_container = "{{ colors.primary_container.default.hex }}",
  on_primary_container = "{{ colors.on_primary_container.default.hex }}",
  secondary = "{{ colors.secondary.default.hex }}",
  on_secondary = "{{ colors.on_secondary.default.hex }}",
  tertiary = "{{ colors.tertiary.default.hex }}",
  error = "{{ colors.error.default.hex }}",
  outline = "{{ colors.outline.default.hex }}",
  on_surface_variant = "{{ colors.on_surface_variant.default.hex }}",
}

hl(0, "Normal", { fg = c.fg, bg = c.bg })
hl(0, "NormalFloat", { fg = c.fg, bg = c.surface })
hl(0, "FloatTitle", { fg = c.primary, bg = c.surface, bold = true })
hl(0, "Cursor", { fg = c.on_primary, bg = c.primary })
hl(0, "CursorLine", { bg = c.surface_low })
hl(0, "CursorLineNr", { fg = c.primary, bold = true })
hl(0, "LineNr", { fg = c.on_surface_variant })
hl(0, "Visual", { bg = c.primary_container, fg = c.on_primary_container })
hl(0, "Search", { fg = c.on_primary, bg = c.primary })
hl(0, "IncSearch", { fg = c.on_secondary, bg = c.secondary })
hl(0, "Substitute", { fg = c.on_primary, bg = c.primary })
hl(0, "StatusLine", { fg = c.fg, bg = c.surface_container })
hl(0, "StatusLineNC", { fg = c.on_surface_variant, bg = c.surface_container })
hl(0, "VertSplit", { fg = c.outline })
hl(0, "WinSeparator", { fg = c.outline })
hl(0, "Pmenu", { fg = c.fg, bg = c.surface })
hl(0, "PmenuSel", { fg = c.on_primary_container, bg = c.primary_container, bold = true })
hl(0, "PmenuSbar", { bg = c.surface_variant })
hl(0, "PmenuThumb", { bg = c.outline })
hl(0, "FloatBorder", { fg = c.outline, bg = c.surface })
hl(0, "MatchParen", { fg = c.primary, bold = true, underline = true })
hl(0, "Comment", { fg = c.on_surface_variant, italic = true })
hl(0, "String", { fg = c.tertiary })
hl(0, "Character", { fg = c.tertiary })
hl(0, "Number", { fg = c.secondary })
hl(0, "Boolean", { fg = c.secondary })
hl(0, "Constant", { fg = c.secondary })
hl(0, "Function", { fg = c.primary })
hl(0, "Identifier", { fg = c.fg })
hl(0, "Statement", { fg = c.primary })
hl(0, "Keyword", { fg = c.primary, bold = true })
hl(0, "Type", { fg = c.secondary })
hl(0, "Operator", { fg = c.on_surface_variant })
hl(0, "PreProc", { fg = c.secondary })
hl(0, "Special", { fg = c.tertiary })
hl(0, "Error", { fg = c.error, bold = true })
hl(0, "DiagnosticError", { fg = c.error })
hl(0, "DiagnosticWarn", { fg = c.secondary })
hl(0, "DiagnosticInfo", { fg = c.primary })
hl(0, "DiagnosticHint", { fg = c.tertiary })
hl(0, "DiffAdd", { fg = c.on_surface, bg = c.surface_container })
hl(0, "DiffChange", { fg = c.on_surface, bg = c.surface_container })
hl(0, "DiffDelete", { fg = c.error, bg = c.surface_container })
hl(0, "@comment", { link = "Comment" })
hl(0, "@string", { link = "String" })
hl(0, "@function", { link = "Function" })
hl(0, "@keyword", { link = "Keyword" })
hl(0, "@type", { link = "Type" })
hl(0, "@variable", { fg = c.fg })
hl(0, "@constant", { link = "Constant" })

vim.g.terminal_color_0 = c.surface
vim.g.terminal_color_1 = c.error
vim.g.terminal_color_2 = c.tertiary
vim.g.terminal_color_3 = c.secondary
vim.g.terminal_color_4 = c.primary
vim.g.terminal_color_5 = c.secondary
vim.g.terminal_color_6 = c.tertiary
vim.g.terminal_color_7 = c.fg
vim.g.terminal_color_8 = c.outline
vim.g.terminal_color_9 = c.error
vim.g.terminal_color_10 = c.tertiary
vim.g.terminal_color_11 = c.secondary
vim.g.terminal_color_12 = c.primary
vim.g.terminal_color_13 = c.secondary
vim.g.terminal_color_14 = c.tertiary
vim.g.terminal_color_15 = c.fg
