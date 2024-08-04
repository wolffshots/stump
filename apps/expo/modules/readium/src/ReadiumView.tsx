import { requireNativeViewManager } from 'expo-modules-core'
import * as React from 'react'

import { ReadiumViewProps } from './Readium.types'

const NativeView: React.ComponentType<ReadiumViewProps> = requireNativeViewManager('Readium')

export default function ReadiumView(props: ReadiumViewProps) {
	return <NativeView {...props} />
}
