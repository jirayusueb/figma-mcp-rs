// FigJam support — editor-type detection and feature guards.

export function isFigjam(): boolean {
  return figma.editorType === "figjam";
}

export function assertNotFigjam(feature: string): void {
  if (isFigjam()) throw new Error(`${feature} is not available in FigJam`);
}
