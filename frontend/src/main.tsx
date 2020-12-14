import React from "react"
import { render } from "react-dom"
import { MaybeModal } from "./components/ModalLink"
import { Routes, BrowserRouterProvider } from "./components/Routes"
import "./style.scss"

const appElement = document.getElementById("root")

function Main() {
	if (!appElement) throw Error("could not find app container")
	return (
		<MaybeModal appElement={appElement}>
			<BrowserRouterProvider>
				<Routes />
			</BrowserRouterProvider>
		</MaybeModal>
	)
}
render(<Main />, appElement)
