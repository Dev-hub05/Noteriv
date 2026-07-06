export type SchemaType =
  | { type: 'str' }
  | { type: 'num' }
  | { type: 'bool' }
  | { type: 'array'; items: SchemaType }
  | { type: 'enum'; values: string[] }
  | { type: 'object'; properties: Record<string, SchemaType>; required: string[] }
  | { type: 'optional'; inner: SchemaType };

export interface RegisteredAction {
  name: string;
  description: string;
  schema: SchemaType;
  handler: (args: any) => Promise<any> | any;
}

class ActionRegistry {
  private actions: Map<string, RegisteredAction> = new Map();

  register(action: RegisteredAction) {
    this.actions.set(action.name, action);
    
    // Sync metadata to Rust Host
    if (typeof window !== 'undefined' && window.electronAPI && (window.electronAPI as any).kriyaRegisterActionMetadata) {
      (window.electronAPI as any).kriyaRegisterActionMetadata({
        name: action.name,
        description: action.description,
        schema: action.schema,
      }).catch((err: any) => console.error("Kriya registry sync failed:", err));
    }
  }

  get(name: string): RegisteredAction | undefined {
    return this.actions.get(name);
  }

  getAll(): RegisteredAction[] {
    return Array.from(this.actions.values());
  }

  async execute(name: string, args: any): Promise<any> {
    const action = this.get(name);
    if (!action) {
      throw new Error(`Action '${name}' not found in TypeScript registry`);
    }
    return await action.handler(args);
  }
}

export const registry = new ActionRegistry();

export function registerAction(
  name: string,
  description: string,
  schema: SchemaType,
  handler: (args: any) => Promise<any> | any
) {
  registry.register({ name, description, schema, handler });
}

// Helper to initialize and register default Noteriv actions wrapper
export function registerDefaultNoterivActions() {
  if (typeof window === 'undefined' || !window.electronAPI) return;

  // 1. read_note
  registerAction(
    "read_note",
    "Reads the content of a markdown note from the vault.",
    {
      type: "object",
      properties: {
        path: { type: "str" }
      },
      required: ["path"]
    },
    async (args) => {
      // Resolve path relative to vault
      const vault = await window.electronAPI.getActiveVault();
      if (!vault) throw new Error("No active vault");
      
      const fullPath = `${vault.path}/${args.path}`;
      const content = await window.electronAPI.readFile(fullPath);
      if (content === null) {
        throw new Error(`Failed to read note at: ${args.path}`);
      }
      return { content };
    }
  );

  // 2. write_note
  registerAction(
    "write_note",
    "Creates or overwrites a markdown note in the vault.",
    {
      type: "object",
      properties: {
        path: { type: "str" },
        content: { type: "str" }
      },
      required: ["path", "content"]
    },
    async (args) => {
      const vault = await window.electronAPI.getActiveVault();
      if (!vault) throw new Error("No active vault");
      
      const fullPath = `${vault.path}/${args.path}`;
      const success = await window.electronAPI.writeFile(fullPath, args.content);
      if (!success) {
        throw new Error(`Failed to write note at: ${args.path}`);
      }
      return { success: true };
    }
  );

  // 3. list_notes
  registerAction(
    "list_notes",
    "Lists markdown files under a specific vault subdirectory.",
    {
      type: "object",
      properties: {
        folder: {
          type: "optional",
          inner: { type: "str" }
        }
      },
      required: []
    },
    async (args) => {
      const vault = await window.electronAPI.getActiveVault();
      if (!vault) throw new Error("No active vault");

      const folderPath = args.folder ? `${vault.path}/${args.folder}` : vault.path;
      const files = await window.electronAPI.listAllFiles(vault.path);
      
      // Filter for markdown files under the target folder
      const prefix = args.folder ? `${args.folder}/` : "";
      const notes = files
        .filter(f => f.relativePath.startsWith(prefix) && f.relativePath.endsWith('.md'))
        .map(f => f.relativePath.slice(prefix.length));

      return { notes };
    }
  );
}
