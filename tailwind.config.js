module.exports = {
    content: ['./templates/**/*.{html,rs}'],
    theme: {
        container: {
            padding: {
                DEFAULT: '1rem',
            },
        },
    },
    daisyui: {
        themes: ['light'],
    },
    plugins: [require('daisyui')],
};
