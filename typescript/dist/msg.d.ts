import { BebopView, BebopRecord } from "bebop";
export declare const BEBOP_SCHEMA: Uint8Array<ArrayBuffer>;
export interface Msg {
    readonly body: string;
    readonly fromId: string;
    readonly id: string;
    readonly toIds: string[];
    readonly type: string;
}
export declare const Msg: ((data: Msg) => Msg & BebopRecord) & {
    encode(record: Msg): Uint8Array;
    encodeInto(record: Msg, view: BebopView): void;
    decode(buffer: Uint8Array): Msg & BebopRecord;
    readFrom(view: BebopView): Msg;
};
//# sourceMappingURL=msg.d.ts.map