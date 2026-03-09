// Demo-only ambient declarations to keep the repo type-checkable even
// before running `npm install`.
//
// In real extension development, `@types/vscode` and `@types/node` provide
// proper typings via node_modules.

declare module "vscode" {
  export const window: any;
  export const workspace: any;
  export const ViewColumn: any;

  export interface OutputChannel {
    appendLine(value: string): void;
    show(preserveFocus?: boolean): void;
  }

  export interface ExtensionContext {
    asAbsolutePath(relativePath: string): string;
    subscriptions: any[];
  }

  export const commands: any;
}

declare module "child_process" {
  export function spawn(command: string, args?: string[], options?: any): any;
}

declare module "path" {
  export function join(...parts: string[]): string;
}

declare module "fs" {
  export function existsSync(p: string): boolean;
}

declare function setTimeout(handler: (...args: any[]) => void, timeout?: number): any;
declare function clearTimeout(handle: any): void;


