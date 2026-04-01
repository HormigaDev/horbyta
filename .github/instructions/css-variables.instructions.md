---
description: "Use when creating or modifying styling in Vue components. Enforces strict usage of CSS variables to maintain theme consistency (Dark/Light mode)."
applyTo: frontend/**/*.vue
---

# Uso Estricto de Variables CSS

Para garantizar que el cambio de temas (Claro/Oscuro) en la aplicación funcione sin errores y evitar definir ajustes en múltiples lugares, sigue siempre esta regla:

- **NUNCA** uses colores quemados en tus bloques `<style>` (ni hexadecimales `#fff`, ni rgb `rgba()`, ni colores nombrados `white`, `black`, `red`).
- **SIEMPRE** utiliza las variables CSS globales previamente definidas en `frontend/src/css/app.scss` (y combinadas con las variables nativas de Quasar si aplica).

## Ejemplos de Variables Permitidas:

- Fondos: `var(--card-bg)`, `var(--q-dark-page)`
- Textos: `var(--text-color)`, `var(--q-primary)`
- Bordes: `var(--border-color)`
- Componentes: `var(--tooltip-bg)`

Si necesitas un color que no existe, agrégalo a la raíz en `app.scss` con su respectiva variante `[data-theme='dark']` o `[data-theme='light']`, pero no lo quemes directamente en el componente `.vue`.
