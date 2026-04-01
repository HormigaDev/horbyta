---
description: "Use when creating or modifying frontend CRUD views, pages, or UI components for entities (like Projects, Tasks, etc). Enforces UI patterns for tables, headers, master-detail screens, and quick actions."
applyTo: frontend/src/pages/**/*.vue, frontend/src/components/**/*.vue
---

# Patrón de Vistas CRUD en Frontend

Para evitar un "frontend Frankenstein", todas las vistas de operaciones CRUD deben seguir estrictamente el siguiente patrón de diseño y usabilidad.

## 1. Vistas de Listado (Data Tables)

- **Componente Base:** Cada vista CRUD debe implementar un componente de tabla personalizado (que actuará como wrapper de `q-table` de Quasar).
- **Header de la Tabla:**
    - Debe contener el título de la entidad (ej. "Proyectos", "Tareas").
    - Diseño compacto y minimalista: no desperdiciar espacio, pero sin saturar la vista.
    - **Quick Actions (Header, Top-Right):** Botones para acciones globales y de fila (Nuevo, Editar, Eliminar).
- **Mobile First:** Todo el diseño de la tabla y del header debe estar optimizado para pantallas móviles primero.

## 2. Vistas Master-Detail (ej. `/projects/:id`)

Define cómo se muestra el detalle de una entidad.

### Cuándo usar Master-Detail vs Modales

- **Master-Detail (Página dedicada):** Sólo para entidades complejas o que requieran renderizar textos largos (ej. Proyectos, Tareas).
- **Modal:** Para entidades simples que no justifican una página entera (ej. Categorías, Tags).

### Estructura de la página Master-Detail

1. **Right Sidebar (Quick Actions Fijas):**
    - Panel lateral derecho con botones de acceso rápido `fixed`.
    - **Formato:** Sólo íconos con `q-tooltip`.
    - **Acciones obligatorias:** Volver a la lista, Editar, Eliminar.
2. **Caja de Especificaciones:**
    - Antes del contenido principal, debe ir una caja/tarjeta con un patrón visual moderno que resuma los metadatos de la entidad (ID, Nombre, Estado, Fechas).
3. **Contenido Principal (Markdown):**
    - Si la entidad tiene descripciones o textos largos, utilizar el componente `MarkdownRenderer`.
    - El renderizador de Markdown será el contenedor que tenga la barra de scroll (no la página entera, de ser necesario).
