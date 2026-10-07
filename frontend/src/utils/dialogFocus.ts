/** Keep sequential keyboard navigation within the currently focused Quasar dialog. */
export function cycleDialogFocus(event: KeyboardEvent): void {
  if (
    event.key !== 'Tab' ||
    event.defaultPrevented ||
    event.altKey ||
    event.ctrlKey ||
    event.metaKey
  )
    return;
  const active = document.activeElement;
  const dialog = active?.closest('.q-dialog--modal');
  if (!dialog) return;
  const controls = Array.from(
    dialog.querySelectorAll<HTMLElement>('button, a[href], input, select, textarea, [tabindex]'),
  ).filter(
    (element) =>
      element.tabIndex >= 0 &&
      !element.matches(':disabled, [aria-disabled="true"]') &&
      !element.closest('[inert], [aria-hidden="true"]') &&
      element.checkVisibility({ opacityProperty: true, visibilityProperty: true }),
  );
  const first = controls[0];
  const last = controls.at(-1);
  if (!first || !last) return;
  if ((!event.shiftKey && active === last) || (event.shiftKey && active === first)) {
    event.preventDefault();
    (event.shiftKey ? last : first).focus();
  }
}
