import { computed } from "mobx"
import React from "react"
import Plot from "react-plotly.js"
import { inflateRawSync } from "zlib"
import { Activity } from "../api"
import { DefaultMap, KeyedSet, totalDuration } from "../util"

export class CategoryChart extends React.Component<{
	events: Activity[]
	tagPrefix: string
	deep: boolean
}> {
	@computed get data() {
		const prefix = this.props.tagPrefix
		const groups = new DefaultMap<string, KeyedSet<Activity>>(
			() => new KeyedSet((e) => e.id),
		)
		for (const event of this.props.events) {
			for (const tag of event.tags) {
				if (tag.startsWith(prefix)) {
					let cat = tag.slice(prefix.length)
					if (!this.props.deep) cat = cat.split("/")[0]
					groups.get(cat).add(event)
				}
			}
		}
		const x = [...groups.keys()]
		const y = [...groups.values()].map(
			(s) => totalDuration([...s]) / 60 / 60,
		)
		return { x, y }
	}
	render(): React.ReactNode {
		return (
			<div>
				<Plot
					data={[
						{
							type: "bar",

							...this.data,
						},
					]}
					layout={{
						width: 700,
						yaxis: {
							title: "Hours",
						},
						height: 400,
						title: `Time spent per ${this.props.tagPrefix}`,
					}}
				/>
			</div>
		)
	}
}
