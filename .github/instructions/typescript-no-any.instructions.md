---
description: "Use when writing or editing TypeScript or Vue files. Enforces zero usage of 'any' across the codebase."
applyTo: "frontend/**/*.ts, frontend/**/*.vue"
---

# Typescript No `any` Policy

**Context**: Esta regla aplica a todos los archivos TypeScript (`*.ts`) y componentes de Vue (`*.vue`).

## REGLAS ESTRICTAS

1. **PROHIBIDO EL USO DE `any`**: JAMÁS debes usar `any` para definir tipos, cambiar el tipo (cast) o como tipo de retorno. El uso de `any` desactiva completamente el sistema de tipos de TypeScript y oculta errores en tiempo de precompilación.
2. **Alternativas requeridas**:
    - Si no conoces el tipo o puede ser literalmente cualquier cosa, utiliza `unknown`. A diferencia de `any`, `unknown` te obliga a realizar comprobaciones de tipo o casteos seguros antes de operar con los datos.
    - Si es un objeto genérico, usa `Record<string, unknown>`.
    - Utiliza Tipos Genéricos (`<T>`) para funciones o componentes que pueden recibir múltiples tipos de clases/datos (por ejemplo, respuestas paginadas o tablas).
    - Crea `interfaces` o `types` específicos si el objeto tiene una estructura conocida.
3. **Excepciones extremas**: Solo se justificará el uso de `any` si una librería de terceros externa sin alternativas correctas de tipado fuerza su uso impidiendo la compilación, y requiere un comentario justificando la situación.
