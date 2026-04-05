import { Component, Input } from '@angular/core';
import { FullCommand } from 'src/app/services/command.service';
import { Param } from 'src/bindings/Param';
import { Syntax } from 'src/bindings/Syntax';
import { ReturnTypeComponent } from "../return-type/return-type.component";
import { Value } from 'src/bindings/Value';
import { ValueComponent } from '../value/value.component';
import { Version } from 'src/bindings/Version';
import { GameVersionComponent } from "../game-version/game-version.component";
import { mapSince } from 'src/app/mapSince';
import { WikiTextComponent } from "../wiki-text/wiki-text.component";

interface ParamInfo {
    name: string;
    type: Value;
    desc?: string;
    since?: { game: string; version: string | Version }[];
}

interface SyntaxInfo {
    signature: string;
    parameters: ParamInfo[];
    since: { game: string; version: string | Version }[];
}

@Component({
    selector: 'app-syntax-viewer',
    standalone: true,
    templateUrl: './syntax-viewer.component.html',
    imports: [ReturnTypeComponent, ValueComponent, GameVersionComponent, WikiTextComponent],
})
export class SyntaxViewerComponent {
    @Input() command!: FullCommand;
    @Input() syntax!: Syntax;

    get syntaxInfo(): SyntaxInfo {
        try {
            return this.parseSyntax(this.command, this.syntax);
        } catch (e) {
            console.error('Error parsing syntax:', e);
            return {
                signature: 'Unable to parse syntax',
                parameters: [],
                since: [],
            };
        }
    }

    private parseSyntax(command: FullCommand, syntax: Syntax): SyntaxInfo {
        if (!syntax || typeof syntax !== 'object') {
            return { signature: String(syntax), parameters: [], since: [] };
        }

        const signature = this.buildSignature(command, syntax);
        const parameters = this.extractParameters(syntax);

        return { signature, parameters, since: mapSince(syntax.since) };
    }

    private buildSignature(command: FullCommand, syntax: Syntax): string {
        if (typeof syntax.call === 'object' && 'Binary' in syntax.call) {
            return `${this.buildSignatureArg(syntax.left!)} <span class="text-orange-700">${command.name}</span> ${this.buildSignatureArg(syntax.right!)}`;
        } else if (syntax.call === 'Nular') {
            return `<span class="text-orange-700">${command.name}</span>`;
        } else if (typeof syntax.call === 'object' && 'Unary' in syntax.call) {
            return `<span class="text-orange-700">${command.name}</span> ${this.buildSignatureArg(syntax.right!)}`;
        }
        return `<span class="text-orange-700">${command.name}</span>`;
    }

    private buildSignatureArg(arg: Param): string {
        if ('Item' in arg && arg.Item) {
            return arg.Item.name;
        }
        if ('Array' in arg && arg.Array) {
            let items = arg.Array.map((item: Param) => this.buildSignatureArg(item)).join(', ');
            return `[${items}]`;
        }
        return `{ Unknown }`;
    }

    private extractParameters(syntax: Syntax): ParamInfo[] {
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
        obj: Param,
        defaultName: string,
    ): ParamInfo[] {
        if (!obj || typeof obj !== 'object') {
            console.error('Invalid parameter object:', obj);
            return [];
        }

        // Look for Item with name and type
        if ('Item' in obj) {
            const item = obj.Item;
            return [{ name: item.name, type: item.type, desc: item.desc || undefined, since: mapSince(item.since) }];
        }

        // Look for Array
        if ('Array' in obj) {
            const items: ParamInfo[] = obj.Array.map((item: Param, index: number) => {
                if ('Item' in item) {
                    const itemData = item.Item;
                    return {
                        name: itemData.name || `${defaultName} ${index + 1}`,
                        type: itemData.type,
                        desc: itemData.desc || undefined,
                        since: mapSince(itemData.since),
                    };
                }
                return { name: `${defaultName} ${index + 1}`, type: 'Unknown', since: [] };
            });
            return items;
        }

        // Look for Infinite
        if ('Infinite' in obj) {
            const items: ParamInfo[] = obj.Infinite.map((item: Param, index: number) => {
                if ('Item' in item) {
                    const itemData = item.Item;
                    return {
                        name: itemData.name || `${defaultName} ${index + 1}`,
                        type: itemData.type,
                        desc: itemData.desc || undefined,
                        since: mapSince(itemData.since)
                    };
                }
                return { name: `${defaultName} ${index + 1}`, type: 'Unknown', since: [] };
            });
            return items;
        }

        console.warn('Unable to extract parameter info from object:', obj);
        return [];
    }
}

