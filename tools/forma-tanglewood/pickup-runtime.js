/** Engine-neutral prototype. Adapter implements attach(id, grip, template), detach(id),
 * setWorldVisible(id, visible). Coordinates: metres, Y up, camera forward +Z.
 * Templates already have pivot-relative vertices: do not subtract grip.pivot twice.
 * No physics, input system, hand animation or renderer is provided here.
 */
export function createPickupController(adapter, templates) {
  const byId = new Map(templates.map(t => [t.id, t]));
  let held = null;
  return {
    get heldId() { return held; },
    equip(id) {
      if (held !== null) throw new Error('Return the held item first');
      const template = byId.get(id);
      if (!template) throw new Error('Unknown pickup ID: ' + id);
      adapter.attach(id, template.grip, template);
      try { adapter.setWorldVisible(id, false); }
      catch (error) { adapter.detach(id); throw error; }
      held = id;
      return template;
    },
    returnToOrigin() {
      if (held === null) return;
      adapter.detach(held);
      adapter.setWorldVisible(held, true);
      held = null;
    }
  };
}
