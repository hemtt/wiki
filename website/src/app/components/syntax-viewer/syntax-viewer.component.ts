import { Component, Input } from '@angular/core';

interface ParamInfo {
    name: string;
    type: string;
}

interface SyntaxInfo {
    signature: string;
    parameters: ParamInfo[];
    returnType: string;
}

@Component({
    selector: 'app-syntax-viewer',
    standalone: true,
    template: `
    <div class="syntax-section">
      <div class="syntax-signature" [innerHTML]="syntaxInfo.signature"></div>

      @if (syntaxInfo.parameters.length > 0) {
        <div class="parameters-section">
          <div class="section-label">Parameters:</div>
          <div class="parameters-list">
            @for (param of syntaxInfo.parameters; track param.name) {
              <div class="parameter-row">
                <span class="param-name">{{ param.name }}</span>
                <span class="param-type">{{ param.type }}</span>
              </div>
            }
          </div>
        </div>
      }

      @if (syntaxInfo.returnType) {
        <div class="return-section">
          <div class="section-label">Return Value:</div>
          <div class="return-type">{{ syntaxInfo.returnType }}</div>
        </div>
      }
    </div>
  `,
    styles: [`
    .syntax-section {
      display: flex;
      flex-direction: column;
      gap: 1.25rem;
    }

    .syntax-signature {
      font-family: 'Monaco', 'Courier New', monospace;
      font-size: 1rem;
      font-weight: 500;
      color: #1f2937;
      background: #f9fafb;
      padding: 0.875rem 1rem;
      border-left: 4px solid #3b82f6;
      border-radius: 0.375rem;
      line-height: 1.6;
      letter-spacing: 0.5px;
    }

    .parameters-section,
    .return-section {
      display: flex;
      flex-direction: column;
      gap: 0.75rem;
    }

    .section-label {
      font-weight: 700;
      font-size: 0.875rem;
      color: #374151;
      text-transform: uppercase;
      letter-spacing: 0.05em;
    }

    .parameters-list {
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
      margin-left: 0.5rem;
    }

    .parameter-row {
      display: grid;
      grid-template-columns: max-content 1fr;
      gap: 1rem;
      align-items: baseline;
      font-family: 'Monaco', 'Courier New', monospace;
      font-size: 0.95rem;
      line-height: 1.5;
    }

    .param-name {
      color: #374151;
      font-weight: 500;
    }

    .param-type {
      color: #7c3aed;
      font-style: italic;
    }

    .return-type {
      font-family: 'Monaco', 'Courier New', monospace;
      font-size: 0.95rem;
      color: #059669;
      font-weight: 500;
      padding: 0.5rem 0.75rem;
      background: #f0fdf4;
      border-left: 3px solid #10b981;
      border-radius: 0.25rem;
    }
  `],
})
export class SyntaxViewerComponent {
    @Input() command: any;
    @Input() syntax: any;

    get syntaxInfo(): SyntaxInfo {
        try {
            return this.parseSyntax(this.command, this.syntax);
        } catch (e) {
            console.error('Error parsing syntax:', e);
            return {
                signature: 'Unable to parse syntax',
                parameters: [],
                returnType: '',
            };
        }
    }

    private parseSyntax(command: any, syntax: any): SyntaxInfo {
        if (!syntax || typeof syntax !== 'object') {
            return { signature: String(syntax), parameters: [], returnType: '' };
        }

        const signature = this.buildSignature(command, syntax);
        const parameters = this.extractParameters(syntax);
        const returnType = this.extractReturnType(syntax);

        return { signature, parameters, returnType };
    }

    private buildSignature(command: any, syntax: any): string {

        console.log('Parsing syntax:', syntax);

        if (syntax.call.Binary) {
            return `${this.buildSignatureArg(syntax.left)} <span class="text-orange-700">${command.name}</span> ${this.buildSignatureArg(syntax.right)}`;
        } else if (syntax.call.Nulary) {
            return `<span class="text-orange-700">${command.name}</span>`;
        } else if (syntax.call.Unary) {
            return `<span class="text-orange-700">${command.name}</span> ${this.buildSignatureArg(syntax.right)}`;
        }
        return `<span class="text-orange-700">${command.name}</span>`;
    }

    private buildSignatureArg(arg: any): string {
        if (arg.Item) {
            return arg.Item.name;
        }
        if (arg.Array) {
            let items = arg.Array.map((item: any) => this.formatType(item)).join(', ');
            return `[${items}]`;
        }
        return `{ Unknown }`;
    }

    private extractParameters(syntax: any): ParamInfo[] {
        const params: ParamInfo[] = [];

        // Check common parameter locations
        if (syntax.left) {
            const leftParam = this.extractParamFromObject(
                syntax.left,
                'left operand',
            );
            if (leftParam) params.push(...leftParam);
        }

        if (syntax.right) {
            const rightParam = this.extractParamFromObject(
                syntax.right,
                'right operand',
            );
            if (rightParam) params.push(...rightParam);
        }

        return params;
    }

    private extractParamFromObject(
        obj: any,
        defaultName: string,
    ): ParamInfo[] {
        if (!obj || typeof obj !== 'object') {
            console.error('Invalid parameter object:', obj);
            return [];
        }

        // Look for Item with name and type
        if (obj.Item) {
            const name = obj.Item.name || defaultName;
            const type = obj.Item.type || this.inferType(obj);
            return [{ name, type }];
        }
        if (obj.Array) {
            let items: ParamInfo[] = [];
            items = obj.Array.map((item: any, index: number) => {
                if (item.Item) {
                    return {
                        name: item.Item.name || `${defaultName} ${index + 1}`,
                        type: item.Item.type || this.inferType(item) || 'Unknown',
                    };
                }
                return { name: `${defaultName} ${index + 1}`, type: 'Unknown' };
            });
            return items;
        }

        // Look for direct type fields
        if (obj.type) {
            const name = obj.name || defaultName;
            return [{ name, type: obj.type }];
        }

        // Look for Binary field
        if (obj.Binary) {
            const parts = Array.isArray(obj.Binary) ? obj.Binary : [obj.Binary];
            const name = defaultName;
            const type = parts.join(' / ');
            return [{ name, type }];
        }

        console.warn('Unable to extract parameter info from object:', obj);
        return [];
    }

    private extractReturnType(syntax: any): string {
        if (syntax.ret) {
            return this.formatType(syntax.ret.type || syntax.ret);
        }
        return '';
    }

    private formatType(value: any): string {
        if (!value) return '';

        console.log('Formatting type for value:', value);

        if (typeof value === 'string') {
            return value;
        }

        if (typeof value === 'object') {
            if (value.Binary && Array.isArray(value.Binary)) {
                return value.Binary[0];
            }
            if (value.Item && value.Item.name) {
                return value.Item.name;
            }
            if (value.ArrayUnsized) {
                return `Array<${this.formatType(value.ArrayUnsized.value)}>`;
            }
            if (value.type) {
                return value.type;
            }
        }

        return '';
    }

    private inferType(obj: any): string {
        if (!obj || typeof obj !== 'object') {
            return typeof obj;
        }

        if (obj.Binary) return 'Value';
        if (obj.Item) return 'Object';
        if (obj.type) return obj.type;

        return 'Unknown';
    }
}

