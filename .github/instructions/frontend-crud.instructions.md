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

## 2. Vistas Master-Detail y Formularios

Define cómo se muestran, crean y editan las entidades que requieren su propia página completa.

### Cuándo usar Master-Detail vs Modales

- **Master-Detail (Página dedicada):** Sólo para entidades complejas o que requieran renderizar textos largos (ej. Proyectos, Tareas).
- **Modal:** Para entidades simples que no justifican una página entera o no contengan campos complejos (ej. Categorías, Tags).

### Estructura de la Página de Visualización (Master-Detail)

Al abrir el enlace `/projects/:id` (sólo modo vista):

1. **Right Sidebar (Quick Actions Fijas):**
    - Panel lateral derecho con botones de acceso rápido (`fixed` o `sticky`).
    - **Acciones obligatorias:** Volver a la lista, Editar, Eliminar.
2. **Caja de Especificaciones:**
    - Antes del contenido principal, debe ir una caja/tarjeta con un patrón visual moderno que resuma los metadatos de la entidad (ID, Nombre, Estado, Fechas).
3. **Contenido Principal (Lectura limpia de Markdown):**
    - Si la entidad tiene descripciones o textos largos, utilizar el componente `MarkdownRenderer`. Únicamente se renderiza la vista final, **sin inputs** y el contenedor debe tener barra de scroll (no la página entera).

### Estructura de la Página de Formularios (Crear/Editar)

Al crear (`/projects/new`) o editar (`/projects/:id/edit`), se debe omitir el sidebar lateral.

1. **Campos principales:**
    - Inputs como Título, Categoría o asginados se muestran en la parte superior en un Layout de cuadrícula normal (full width responsivo).
2. **Campos de Texto Complejo (Markdown):**
    - Layout "Side-by-Side" (Lado a lado). Mitad izquierda un input `textarea` gigante, mitad derecha usando `MarkdownRenderer` a tiempo real (solo lectura visual).
3. **Botones de Guardado / Acciones:**
    - A diferencia del Master-Detail visual, en los formularios las acciones de "Guardar" y "Cancelar" **se colocan de forma clásica al final del formulario** en la parte inferior, NO en un panel lateral.
