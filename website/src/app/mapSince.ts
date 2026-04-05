import { Since } from "src/bindings/Since";
import { Version } from "src/bindings/Version";

export function mapSince(since?: Since | null): { game: string; version: string | Version }[] {
    return since ? Object.keys(since).filter(game => since![game as keyof Since] !== null).map(game => ({ game, version: since![game as keyof Since]! })) : [];
}
