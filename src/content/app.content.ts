import { t, type Dictionary } from "intlayer";

const appContent = {
    key: "app",
    content: {
        viteLogo: t({
            en: "Vite logo",
            fr: "Logo Vite",
            es: "Logo Vite",
        }),
        reactLogo: t({
            en: "React logo",
            fr: "Logo React",
            es: "Logo React",
        }),

        title: "Vite + React",

        count: t({
            en: "count is ",
            fr: "le compte est ",
            es: "el recuento es ",
        }),

        readTheDocs: t({
            en: "Click on the Vite and React logos to learn more",
            fr: "Cliquez sur les logos Vite et React pour en savoir plus",
            es: "Haga clic en los logotipos de Vite y React para obtener más información",
        }),
    },
} satisfies Dictionary;

export default appContent;