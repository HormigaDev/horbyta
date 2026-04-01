<template>
  <q-page padding>
    <div class="row q-col-gutter-md">
      <div class="col-12">
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
import MarkdownRenderer from 'src/components/MarkdownRenderer.vue';

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
