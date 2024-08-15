import type { CompletionContext } from '@codemirror/autocomplete';

const openTag = '${[ ';
const closeTag = ' ]}';

export interface TwigCompletionOption {
  name: string;
  label: string;
  type: 'function' | 'variable' | 'unknown';
  value: string | null;
  onClick?: (rawTag: string) => void;
}

export interface TwigCompletionConfig {
  options: TwigCompletionOption[];
}

const MIN_MATCH_VAR = 1;
const MIN_MATCH_NAME = 2;

export function twigCompletion({ options }: TwigCompletionConfig) {
  return function completions(context: CompletionContext) {
    const toStartOfName = context.matchBefore(/\w*/);
    const toStartOfVariable = context.matchBefore(/\$\{?\[?\s*\w*/);
    const toMatch = toStartOfVariable ?? toStartOfName ?? null;

    if (toMatch === null) return null;

    const matchLen = toMatch.to - toMatch.from;

    const failedVarLen = toStartOfVariable !== null && matchLen < MIN_MATCH_VAR;
    if (failedVarLen && !context.explicit) {
      return null;
    }

    const failedNameLen = toStartOfVariable === null && matchLen < MIN_MATCH_NAME;
    if (failedNameLen && !context.explicit) {
      return null;
    }

    // TODO: Figure out how to make autocomplete stay open if opened explicitly. It sucks when you explicitly
    //  open it, then it closes when you type the next character.
    return {
      validFor: () => true, // Not really sure why this is all it needs
      from: toMatch.from,
      options: options
        .filter((v) => v.name.trim())
        .map((v) => {
          const innerLabel = v.type === 'function' ? `${v.name}()` : v.name;
          const tagSyntax = openTag + innerLabel + closeTag;
          return {
            label: innerLabel,
            apply: tagSyntax,
            type: v.type === 'variable' ? 'variable' : 'function',
            matchLen: matchLen,
          };
        })
        // Filter out exact matches
        .filter((o) => o.label !== toMatch.text),
    };
  };
}
