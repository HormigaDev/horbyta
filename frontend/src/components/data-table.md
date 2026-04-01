# DataTable Component

Un componente Wrapper de alta flexibilidad basado en `q-table` de Quasar, potenciado ("con esteroides") para un uso masivo y consistente a lo largo de toda la aplicación, cumpliendo estrictamente con el patrón Mobile-First y respetando las directrices de UI.

## Filosofía de Diseño

Este componente **No gestiona llamadas HTTP (Axios/Fetch)**. Su principal propósito es ser una vista que se alimenta reactivamente de props y comunica los intentados de acciones (pagination, crud) para delegarlos al componente padre.
Maneja las responsabilidades UI de forma unificada:

- Seleccionar y recordar las **columnas visibles** a través de Pinia.
- Configurar y recordar los ajustes de **paginación** a través de Pinia (`rowsPerPage`, `sortBy`, `descending`).
- Implementar los botones principales del CRUD (Nuevo, Editar, Eliminar, Recargar).
- Proveer soporte natural para multi-selección o selección sencilla a través de casillas de verificación (checkboxes).

## Props (Parámetros) Principales

- `tableId` _(String, Obligatorio)_: Identificador único usado por el Store de Pinia para recordar la configuración del usuario en localStorage.
- `title` _(String)_: El título que se mostrará en el Header.
- `columns` _(Array)_: Mismo formato de array de objetos `{ name, label, field, sortable... }` de Quasar.
- `rows` _(Array)_: Array de datos/registros renderizar.
- `totalItems` _(Number)_: El recuento real desde la base de datos (para gestionar el offset remotamente).
- `rowKey` _(String, Default: `'id'`)_: El identificador único de cada fila.
- `loading` _(Boolean)_: Indicador visual de si el componente superior sigue resolviendo datos HTTP.
- `selectionMode` _(String: `'multiple'` | `'single'` | `'none'`)_: Default a `'multiple'`.
- `hideCreate`, `hideEdit`, `hideDelete`, `hideRefresh` _(Boolean)_: Switches opcionales para apagar cualquiera de los quick actions fijos por defecto.

## Emits (Eventos)

- `@request` _(Event Payload de Quasar Paginator)_: Se emite tras interacción de paginado/orden de columna o al montarse por primera vez. **Úsalo para hacer tus peticiones al Backend o Composable.**
- `@create`: Invocado al presionar el botón "Nuevo" en el Header.
- `@edit`: Invocado al presionar el ícono Editar. Emite el **Array de IDs** que fueron seleccionados (incluso si es 1 solo).
- `@delete`: Invocado al presionar el ícono Eliminar. Emite el **Array de IDs**.
- `@refresh`: Invocado con el botón refresh.
- `@selection-change`: Te permite suscribirte a los cambios en el array temporal de elementos checkbox.
- `@sort`: Emite un objeto `{ sortBy, descending }` si hubo un cambio estricto de columna a ordenar o de dirección del ordenado (ascendente a descendente), permitiendo al componente padre saber puntualmente que el criterio de orden ha fluido de forma distinta a la página.

## Uso de Slots (Extensibilidad Máxima)

Al heredar de `$slots`, `DataTable` te da acceso natural a los templated nativos subyacentes de Quasar `q-table`:

- **Columnas Personalizadas:** Transforma el renderizado interior de cualquier celda usando el slot pattern `#body-cell-[name]`.
- **Top Actions `#top-actions`:** Inyecta tus propios botones extras de encabezado antes o después de la selección de columnas y botones default (ejemplo botón imprimir, exportar).
- **Filtros Personalizados `#filters`:** Por si quieres incrustar un panel de búsqueda o combos select antes del render de las filas con diseño flex.

### Ejemplo de Implementación

```vue
<DataTable
  tableId="my_unique_projects"
  title="Gestión de Proyectos"
  :columns="columns"
  :rows="projects"
  :totalItems="150"
  @request="fetchProjects"
  @create="openCreateModal"
  @edit="editSelected"
  @delete="confirmDestruction"
>
  <template #body-cell-status="props">
    <q-td :props="props">
      <q-badge color="primary">{{ props.row.status }}</q-badge>
    </q-td>
  </template>
</DataTable>
```
