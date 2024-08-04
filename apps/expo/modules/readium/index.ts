import { EventEmitter, NativeModulesProxy, Subscription } from 'expo-modules-core'

import { ChangeEventPayload, ReadiumViewProps } from './src/Readium.types'
// Import the native module. On web, it will be resolved to Readium.web.ts
// and on native platforms to Readium.ts
import ReadiumModule from './src/ReadiumModule'
import ReadiumView from './src/ReadiumView'

// Get the native constant value.
export const PI = ReadiumModule.PI

export function hello(): string {
	return ReadiumModule.hello()
}

export async function setValueAsync(value: string) {
	return await ReadiumModule.setValueAsync(value)
}

const emitter = new EventEmitter(ReadiumModule ?? NativeModulesProxy.Readium)

export function addChangeListener(listener: (event: ChangeEventPayload) => void): Subscription {
	return emitter.addListener<ChangeEventPayload>('onChange', listener)
}

export { ChangeEventPayload, ReadiumView, ReadiumViewProps }
