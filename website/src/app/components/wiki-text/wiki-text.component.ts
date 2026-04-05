import { Component, Input, OnChanges, signal, SimpleChanges } from '@angular/core';
import { CommonModule } from '@angular/common';
import { WikiContentRendererComponent } from './wiki-content-renderer.component';

declare const Parser: any;

@Component({
    selector: 'app-wiki-text',
    standalone: true,
    imports: [CommonModule, WikiContentRendererComponent],
    template: `<div>
        @if (ast()) {
            <app-wiki-content-renderer [nodes]="ast()!"></app-wiki-content-renderer>
        } @else {
            <p>Loading...</p>
        }
    </div>`,
})
export class WikiTextComponent implements OnChanges {
    @Input() source!: string;
    ast = signal<any[] | null>(null);

    ngOnChanges(changes: SimpleChanges) {
        if (changes['source'] && this.source) {
            this.updateAST();
        }
    }

    private async updateAST() {
        try {
            let ast = Parser.parse(this.source);
            ast = this.createQuotes(ast.childNodes);
            ast = this.createLists(ast);
            this.ast.set(ast);
            console.log('Parsed AST:', ast);
        } catch (error) {
            console.error('Failed to parse:', error);
        }
    }

    /// Move all items between quote nodes into the first quote's childNodes
    private createQuotes(nodes: any[]): any[] {
        const result: any[] = [];
        let i = 0;

        while (i < nodes.length) {
            const node = nodes[i];
            if (node.type === 'quote') {
                const quoteNodes: any[] = [];
                let j = i + 1;

                // Collect siblings into the quote until we hit the next quote
                while (j < nodes.length) {
                    const sibling = nodes[j];

                    if (sibling.type !== 'quote') {
                        sibling.childNodes = this.createQuotes(sibling.childNodes || []);
                        quoteNodes.push(sibling);
                        j++;
                    } else {
                        break;
                    }
                }

                // Move all collected nodes into the first quote's childNodes
                node.childNodes = quoteNodes;
                result.push(node);
                i = j + 1; // Skip processed nodes and the closing quote
            } else {
                node.childNodes = this.createQuotes(node.childNodes || []);
                result.push(node);
                i++;
            }
        }

        return result;
    }

    private createLists(nodes: any[]): any[] {
        const result: any[] = [];
        let i = 0;

        while (i < nodes.length) {
            const node = nodes[i];
            if (node.type === 'list') {
                const childNodes: any[] = [];
                let j = i + 1;

                // Collect siblings into the list until we hit a newline
                while (j < nodes.length) {
                    const sibling = nodes[j];

                    // Check if this is a text node with a newline
                    if (sibling.type === 'text' && sibling.data.includes('\n')) {
                        // Split the text node at the first newline
                        const newlineIndex = sibling.data.indexOf('\n');
                        const beforeNewline = sibling.data.substring(0, newlineIndex);
                        const afterNewline = sibling.data.substring(newlineIndex + 1);

                        // Add text before newline to list children if it exists
                        if (beforeNewline) {
                            sibling.data = beforeNewline;
                            childNodes.push(sibling);
                        }

                        // Add the list with its collected children to result
                        node.childNodes = childNodes;
                        result.push(node);

                        // Add text after newline as a sibling if it exists
                        if (afterNewline) {
                            sibling.data = afterNewline;
                            result.push(sibling);
                        }

                        i = j + 1;
                        break;
                    } else {
                        // Add this sibling to the list's children
                        childNodes.push(sibling);
                        j++;
                    }
                }

                // If we reached the end without finding a newline
                if (j === nodes.length) {
                    node.childNodes = childNodes;
                    result.push(node);
                    i = j;
                }
            } else {
                node.childNodes = this.createLists(node.childNodes || []);
                result.push(node);
                i++;
            }
        }

        // Create list-groups for consecutive lists
        const finalResult: any[] = [];
        let currentListGroup: any[] = [];

        for (const node of result) {
            if (node.type === 'list') {
                currentListGroup.push(node);
            } else {
                if (currentListGroup.length > 0) {
                    finalResult.push({ type: 'list-group', childNodes: currentListGroup });
                    currentListGroup = [];
                }
                finalResult.push(node);
            }
        }

        // If we ended with a list group, add it to final result
        if (currentListGroup.length > 0) {
            finalResult.push({ type: 'list-group', childNodes: currentListGroup });
        }

        return finalResult;
    }
}
