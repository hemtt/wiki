import { Component, Input } from '@angular/core';
import { Version } from 'src/bindings/Version';

@Component({
    selector: 'app-game-version',
    standalone: true,
    templateUrl: './game-version.component.html',
})
export class GameVersionComponent {
    @Input() game!: string;
    @Input() version!: Version | string;
    @Input() slim = false;

    get image(): string {
        return `/assets/games/${this.game.toLowerCase()}.webp`;
    }

    get gameName(): string {
        switch (this.game) {
            case 'ofp':
                return 'Operation Flashpoint';
            case 'ofpe':
                return 'Operation Flashpoint: Elite';
            case 'arma1':
                return 'Arma: Armed Assault';
            case 'arma2':
                return 'Arma 2';
            case 'arma2oa':
                return 'Arma 2: Operation Arrowhead';
            case 'tkoh':
                return 'Take On Helicopters';
            case 'arma3':
                return 'Arma 3';
            case 'argo':
                return 'Argo';
            default:
                return this.game;
        }
    }

    get tooltip(): string {
        return `${this.gameName} ${this.versionText}`;
    }

    get versionText(): string {
        if (typeof this.version === 'string') {
            return this.version;
        }
        let minor = this.version?.minor !== undefined ? this.version.minor.toFixed(2) : '';
        return `${this.version?.major ?? ''}${this.version ? '.' : ''}${minor}`;
    }
}

