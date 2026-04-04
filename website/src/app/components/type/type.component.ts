import { Component, Input } from '@angular/core';

@Component({
    selector: 'app-type',
    standalone: true,
    template: `
    <a target="_blank" [href]="'https://community.bistudio.com/wiki/' + getLink" class="z-0 hover:z-50 relative group text-violet-600">
        <span class="italic">{{ display }}</span>
        @if (getHint()) {
        <div class="absolute left-1/2 -translate-x-1/2 bottom-full mb-2
                w-max p-3 rounded-xl shadow-lg
                bg-gray-900 text-white text-sm
                opacity-0 group-hover:opacity-100
                pointer-events-none transition">
            <div [innerHTML]="getHint()"></div>
        </div>
        }
    </a>
    `
})
export class TypeComponent {
    @Input() type!: string;

    get display(): string {
        switch (this.type) {
            case 'ArraySized':
            case 'ArrayUnsized':
                return 'Array';
            case 'ArrayColor':
                return 'Color';
            case 'ArrayColorRgb':
                return 'Color (RGB)';
            case 'ArrayColorRgba':
                return 'Color (RGBA)';
            case 'Position2d':
                return 'Position (2D)';
            case 'Position3d':
                return 'Position (3D)';
            case 'Position3dASL':
                return 'Position (ASL)';
            case 'Position3dASLW':
                return 'Position (ASLW)';
            case 'Position3dATL':
                return 'Position (ATL)';
            case 'Position3dAGL':
                return 'Position (AGL)';
            case 'Position3dAGLS':
                return 'Position (AGLS)';
            case 'Position3dRelative':
                return 'Position (Relative)';
            case 'Position3dWorld':
                return 'Position (World)';
            case 'TurretPath':
                return 'Turret Path';
            case 'TreeViewPath':
                return 'Tree View Path';
            case 'HashMapKey':
                return 'HashMap Key';
            default:
                return this.type;
        }
    }

    get getLink(): string {
        switch (this.type) {
            case 'ArraySized':
            case 'ArrayUnsized':
                return 'Array';
            case 'ArrayColor':
            case 'ArrayColorRgb':
            case 'ArrayColorRgba':
                return 'Color';
            case 'Position2d':
                return 'Position#Position2d';
            case 'Position3d':
                return 'Position#Position3d';
            case 'Position3dASL':
                return 'Position#PositionASL';
            case 'Position3dASLW':
                return 'Position#PositionASLW';
            case 'Position3dATL':
                return 'Position#PositionATL';
            case 'Position3dAGL':
                return 'Position#PositionAGL';
            case 'Position3dAGLS':
                return 'Position#PositionAGLS';
            case 'Position3dRelative':
                return 'Position#PositionRelative';
            case 'Position3dWorld':
                return 'Position#PositionWorld';
            case 'TurretPath':
                return 'Turret_Path';
            case 'TreeViewPath':
                return 'Arma_3:_Tree_View_Path';
            default:
                return this.type;
        }
    }

    getHint(): string {
        switch (this.type) {
            case 'ArraySized':
                return 'An array with specified set of elements. Each element typically has a name, type, and optional description.';
            case 'ArrayUnsized':
                return 'An array with an unspecified size.';
            case 'ArrayColor':
                return 'An array representing a color, typically in RGB format (e.g., [255, 0, 0] for red).';
            case 'ArrayColorRgb':
                return 'An array representing a color in RGB format (e.g., [255, 0, 0] for red).';
            case 'ArrayColorRgba':
                return 'An array representing a color in RGBA format (e.g., [255, 0, 0, 1] for red with full opacity).';
            case 'Position2d':
                return 'A 2D position, represented as an array of two numbers [x, y].';
            case 'Position3d':
                return 'A 3D position, represented as an array of three numbers [x, y, z].';
            case 'Position3dASL':
                return 'A 3D position relative to the average sea level (ASL), represented as an array of three numbers [x, y, z].';
            case 'Position3dASLW':
                return 'A 3D position relative to the average sea level with wind (ASLW), represented as an array of three numbers [x, y, z].';
            case 'Position3dATL':
                return 'A 3D position relative to the terrain level (ATL), represented as an array of three numbers [x, y, z].';
            case 'Position3dAGL':
                return 'A 3D position relative to the ground level (AGL), represented as an array of three numbers [x, y, z].';
            case 'Position3dAGLS':
                return 'A 3D position relative to the ground level with slope (AGLS), represented as an array of three numbers [x, y, z].';
            case 'Position3dRelative':
                return 'A 3D position relative to a reference point, represented as an array of three numbers [x, y, z].';
            case 'Position3dWorld':
                return 'A 3D position in world coordinates, represented as an array of three numbers [x, y, z].';
            case 'TurretPath':
                return `A path to a turret in a vehicle, represented as an array of numbers indicating the hierarchy of the turret.
                <div class="grid grid-cols-[max-content_1fr] gap-2 mt-2">
                    <div class="font-medium">Path</div>
                    <div>Position</div>
                    <div>[-1]</div><div>driver seat</div>
                    <div>[0]</div><div>first turret (usually gunner)</div>
                    <div>[1]</div><div>second turret</div>
                    <div>[0,0]</div><div>first turret's first turret</div>
                    <div>[0,1]</div><div>first turret's second turret</div>
                    <div>[1,0]</div><div>second turret's first turret, etc. </div>
                </div>
                `;
            case 'TreeViewPath':
                return 'A path in a tree view structure, represented as an array of numbers indicating the hierarchy. <img src="assets/treeview.jpg" alt="Tree View Example" class="mt-2 rounded">';
            case 'HashMapKey':
                return `A virtual compound type containing all the possible types that can be used as keys in a HashMap.
                <div class="flex flex-col gap-2 mt-2">
                    <p><span class="text-violet-200 italic">HashMapKey</span> can be any of the following types:</p>
                    <ul class="list-disc list-inside">
                        <li class="text-orange-300">Number</li>
                        <li class="text-orange-300">Boolean</li>
                        <li class="text-orange-300">String</li>
                        <li class="text-orange-300">Code</li>
                        <li class="text-orange-300">Side</li>
                        <li class="text-orange-300">Config</li>
                        <li class="text-orange-300">Namespace</li>
                        <li class="text-orange-300">NaN</li>
                        <li class="text-orange-300">Array</li>
                    </ul>
                </div>
                `;

            case 'Control':
                return 'A user interface control element, such as a button or slider.';
            case 'Display':
                return 'A user interface display element, such as a dialog or menu.';
            case 'Object':
                return 'An object in the game world, such as a unit (player or AI), vehicle, building, or game logic, or rope.';

            case 'Number':
                return 'A numeric value, which can be an integer or a floating-point number.';
            case 'Boolean':
                return 'A boolean value, which can be either true or false.';
            case 'String':
                return 'A sequence of characters';
            case 'Anything':
                return 'Any type of value is accepted.';
            case 'Nothing':
                return 'No value is accepted or returned.';

            default:
                return '';
        }
    }
}
