/**
 * Kriya MCP Export — converts the TypeScript action registry into MCP tool definitions.
 *
 * This allows external MCP clients (Claude Desktop, Cursor, etc.) to discover
 * Noteriv's Kriya actions as MCP tools with properly typed input schemas.
 */

import { registry, SchemaType } from './kriya-registry';

export interface MCPToolDefinition {
  name: string;
  description: string;
  inputSchema: {
    type: 'object';
    properties: Record<string, any>;
    required: string[];
  };
}

function schemaTypeToJsonSchema(schema: SchemaType): any {
  switch (schema.type) {
    case 'str':
      return { type: 'string' };
    case 'num':
      return { type: 'number' };
    case 'bool':
      return { type: 'boolean' };
    case 'array':
      return { type: 'array', items: schemaTypeToJsonSchema(schema.items) };
    case 'enum':
      return { type: 'string', enum: schema.values };
    case 'optional':
      return schemaTypeToJsonSchema(schema.inner);
    case 'object': {
      const properties: Record<string, any> = {};
      for (const [key, val] of Object.entries(schema.properties)) {
        properties[key] = schemaTypeToJsonSchema(val);
      }
      return {
        type: 'object',
        properties,
        required: schema.required,
      };
    }
    default:
      return { type: 'string' };
  }
}

/**
 * Export all registered Kriya actions as MCP tool definitions.
 */
export function exportMCPTools(): MCPToolDefinition[] {
  const actions = registry.getAll();
  return actions.map((action) => {
    const jsonSchema = schemaTypeToJsonSchema(action.schema);
    return {
      name: action.name,
      description: action.description,
      inputSchema: {
        type: 'object' as const,
        properties: jsonSchema.properties || {},
        required: jsonSchema.required || [],
      },
    };
  });
}

/**
 * Export as a JSON string suitable for MCP server tool listing responses.
 */
export function exportMCPToolsJSON(): string {
  return JSON.stringify({ tools: exportMCPTools() }, null, 2);
}
