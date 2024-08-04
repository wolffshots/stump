import * as React from 'react'

import { ReadiumViewProps } from './Readium.types'

export default function ReadiumView(props: ReadiumViewProps) {
	return (
		<div>
			<span>{props.name}</span>
		</div>
	)
}
