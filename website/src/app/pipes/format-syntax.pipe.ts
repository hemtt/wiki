import { Pipe, PipeTransform } from '@angular/core';

interface SyntaxVariant {
  operand?: string;
  returns?: string;
  [key: string]: any;
}

@Pipe({
  name: 'formatSyntax',
  standalone: true,
})
export class FormatSyntaxPipe implements PipeTransform {
  transform(syntax: SyntaxVariant | any): string {
    if (!syntax || typeof syntax !== 'object') {
      return JSON.stringify(syntax, null, 2);
    }

    // Try to format as a command syntax variant
    const parts: string[] = [];

    // Command name with operand
    if (syntax.operand) {
      parts.push(`command ${syntax.operand}`);
    }

    // Return type
    if (syntax.returns) {
      const returnDisplay = Array.isArray(syntax.returns)
        ? `[${syntax.returns.join(' or ')}]`
        : syntax.returns;
      parts.push(`→ ${returnDisplay}`);
    }

    if (parts.length > 0) {
      return parts.join('\n');
    }

    // Fallback to pretty JSON for complex structures
    return JSON.stringify(syntax, null, 2);
  }
}
