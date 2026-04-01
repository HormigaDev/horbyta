---
description: "Use when working with Vue or TypeScript files. Strictly prohibits the use of terminal tools (run_in_terminal) to check errors, lint, or read files."
applyTo: "frontend/**/*.ts, frontend/**/*.vue"
---

# Prohibición de Terminal para Typescript y Vue

**Context**: Esta regla aplica a todas las interacciones del agente IA con archivos del frontend (`*.ts` y `*.vue`).

## REGLAS ESTRICTAS DEL AGENTE

1. **PROHIBIDO USAR `run_in_terminal` PARA DIAGNÓSTICO FRONTEND**: Bajo ninguna circunstancia utilizarás la herramienta de terminal para:
    - Leer archivos (`cat`, `head`, `less`, `grep`).
    - Comprobar tipos (`npx tsc`, `tsc`).
    - Ejecutar linters (`npm run lint`, `eslint`).
    - Inspeccionar el código de Vue o TypeScript.
2. **Uso Obligatorio de Herramientas Nativas (API/Workspace)**:
    - Para saber el contenido de un archivo: Usar `read_file`.
    - Para buscar texto, componentes o variables: Usar `grep_search`.
    - Para comprobar errores de precompilación, tipado o linting: Usar estrictamente la herramienta `get_errors` indicando las rutas o directorios exactos.
3. **Cero Excepciones**: No intentar evadir esta instrucción. Todo diagnóstico de errores en el código fuente debe hacerse a través del Language Server de VS Code mediante las API habilitadas para el agente.
