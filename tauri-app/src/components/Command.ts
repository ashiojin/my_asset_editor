import { Result } from '@praha/byethrow'
export enum GraphCommandType {
    PlayRepeat,
    Stop,
    Weight,
}
export interface GraphCommnadCommonData {
    selected: string,
}
export type PlayCommand = GraphCommnadCommonData & {
    type: GraphCommandType.PlayRepeat
}
export type StopCommand = GraphCommnadCommonData & {
    type: GraphCommandType.Stop
}
export type WeightCommand = GraphCommnadCommonData & {
    type: GraphCommandType.Weight,
    weight: number,
}
export type GraphCommand = PlayCommand | StopCommand | WeightCommand


export function getDefault(type: GraphCommandType): GraphCommand {
    switch (type) {
        case GraphCommandType.PlayRepeat:
            return { type, selected: '' }
        case GraphCommandType.Weight:
            return { type, selected: '', weight: 1.0 }
        case GraphCommandType.Stop:
            return { type, selected: '' }
    }
}
export enum CheckCommandValidationErrorType {
    NoNodeSelected,
    IllegalWeight,
}
export type CheckCommandValidationError = {
    type: CheckCommandValidationErrorType,
    message: string,
    command: GraphCommand,
}
export function check_command(command: GraphCommand): Result.Result<true, CheckCommandValidationError[]> {
    const errors = []
    // common validations
    if (command.selected === '') {
        errors.push({
            type: CheckCommandValidationErrorType.NoNodeSelected,
            message: 'Need to select node',
            command,
        })
    }

    // per command validations
    switch (command.type) {
        case GraphCommandType.PlayRepeat:
        case GraphCommandType.Stop:
            // no check
            break
        case GraphCommandType.Weight:
            if (command.weight < 0.0 || command.weight > 1.0) {
                errors.push({
                    type: CheckCommandValidationErrorType.IllegalWeight,
                    message: 'Weight must be between 0.0 and 1.0',
                    command,
                })
            }
            break
    }

    if (errors.length > 0) {
        return Result.fail(errors)
    } else {
        return Result.succeed(true)
    }
}
