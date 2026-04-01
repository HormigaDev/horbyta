<template>
  <q-page padding>
    <div class="row q-col-gutter-md">
      <!-- DataTable Mocks Section -->
      <div class="col-12">
        <h4 class="text-h4 text-weight-bold q-mt-none text-grey-8">Demo: Data Table (CRUD)</h4>
      </div>

      <div class="col-12">
        <DataTable
          tableId="dashboard_projects_table"
          title="Gestión de Proyectos"
          :columns="mockColumns"
          :rows="currentRows"
          :totalItems="currentRows.length"
          rowKey="id"
          selectionMode="multiple"
          @create="handleCreate"
          @edit="handleEdit"
          @delete="handleDelete"
          @refresh="handleRefresh"
        >
          <!-- Custom action en la esquina superior -->
          <template #top-actions>
            <q-btn flat round dense icon="print" color="grey-7">
              <q-tooltip>Imprimir</q-tooltip>
            </q-btn>
          </template>

          <!-- Celda Status customizada -->
          <template #body-cell-status="props">
            <q-td :props="props">
              <q-chip
                :color="
                  props.row.status === 'Completado'
                    ? 'positive'
                    : props.row.status === 'En Progreso'
                      ? 'info'
                      : 'warning'
                "
                text-color="white"
                size="sm"
              >
                {{ props.row.status }}
              </q-chip>
            </q-td>
          </template>
        </DataTable>
      </div>

      <!-- ...resto markdown demo -->
      <div class="col-12 q-mt-xl">
        <h4 class="text-h4 text-weight-bold q-mt-none text-grey-8">Demo: Editor Markdown</h4>
      </div>

      <div class="col-12 col-md-6">
        <q-card bordered flat>
          <q-card-section>
            <div class="text-h6 q-mb-md">Modo Edición</div>
            <MarkdownRenderer v-model="markdownText" :edit-mode="true" />
          </q-card-section>
        </q-card>
      </div>

      <div class="col-12 col-md-6">
        <q-card bordered flat>
          <q-card-section>
            <div class="text-h6 q-mb-md">Modo Vista Previa</div>
            <MarkdownRenderer :model-value="markdownText" :edit-mode="false" />
          </q-card-section>
        </q-card>
      </div>
    </div>
  </q-page>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useQuasar } from 'quasar';
import MarkdownRenderer from 'src/components/MarkdownRenderer.vue';
import DataTable from 'src/components/DataTable.vue';
import { mockColumns, mockRows } from 'src/data/mockDataTable';

const $q = useQuasar();
const currentRows = ref([...mockRows]);

// Funciones para probar el DataTable
const handleRefresh = () => {
  $q.notify({ type: 'info', message: 'Tabla Recargada...' });
};
const handleCreate = () => {
  $q.notify({ type: 'positive', message: 'Abriendo Modal Crear...' });
};
const handleEdit = (ids: string[]) => {
  $q.notify({ type: 'warning', message: `Editando los registros: ${ids.join(', ')}` });
};
const handleDelete = (ids: string[]) => {
  $q.dialog({
    title: 'Confirmar Eliminación',
    message: `¿Estás seguro de que deseas eliminar los ${ids.length} elementos seleccionados?`,
    cancel: true,
    persistent: true,
  }).onOk(() => {
    currentRows.value = currentRows.value.filter((row) => !ids.includes(row.id));
    $q.notify({ type: 'negative', message: 'Registros Eliminados.' });
  });
};

const markdownText = ref(`
# Encabezado 1
## Encabezado 2
### Encabezado 3

Este es un texto normal con **negrita** y con *cursiva*. 
También puedes combinar ***ambas***.

> Esto es una cita.
> Puede tener múltiples líneas.

Lista desordenada:
- Elemento 1
- Elemento 2
  - Subelemento A
  - Subelemento B

Lista ordenada:
1. Primero
2. Segundo
3. Tercero

Código en línea: \`const a = "Hola";\`

Bloque de código:
\`\`\`ts
function saludar(nombre: string) {
  console.log(\`Hola, \${nombre}!\`);
}
\`\`\`

Un [enlace a Google](https://google.com)

| Cabecera 1 | Cabecera 2 |
| --- | --- |
| Fila 1 Col 1 | Fila 1 Col 2 |
| Fila 2 Col 1 | Fila 2 Col 2 |

---

Línea divisoria arriba.
`);
</script>
