Promise.all([
    import("dune2"),
    fetch("/dune2.rc"),
]).then(([
    { Dune2Resources },
    res,
]) => {
    return res.arrayBuffer().then(buf => {
        const resources = Dune2Resources.load(new Uint8Array(buf))

        console.log(resources.getSprites())

        // const imageData = resources.getTile("tiles_16x16", 350, "harkonnen", 4)
        // const imageData = resources.getTilemap(10, "harkonnen", 4)
        const imageData = resources.getSpriteFrame("Starport", 3, "ordos", 4)

        return createImageBitmap(imageData)
    })
}).then(imageBitmap => {
    const canvas = document.querySelector("#canvas") as HTMLCanvasElement

    canvas.width = imageBitmap.width
    canvas.height = imageBitmap.height

    const ctx = canvas.getContext("2d")
    if (ctx !== null) {
        ctx.drawImage(imageBitmap, 0, 0)
    }
})
