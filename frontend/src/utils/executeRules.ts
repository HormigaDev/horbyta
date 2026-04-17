import type { Rule, RuleErrorState } from 'src/types';
import type { Ref } from 'vue';
import type { ComposerTranslation } from 'vue-i18n';

/**
 * Ejecuta las reglas de un componente y devuelve true si encontró algún error y false si todo está correcto
 * @param state - El estado de la regla del componente
 * @param rules - La lista de reglas a ejecutar
 * @param prop - La propiedad usada para validar contra las reglas
 * @returns {boolean} - true si encontró algún error, false si todo está correcto
 */
export function executeRules<T>(
  state: Ref<RuleErrorState>,
  rules: Array<Rule<T>>,
  prop: T,
  t: ComposerTranslation,
): boolean {
  for (const rule of rules) {
    const msg = rule(prop, t);
    if (msg) {
      state.value.error = true;
      state.value.message = msg;
      return true;
    }
  }
  return false;
}
