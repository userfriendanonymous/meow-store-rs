
async function loadUserByName() {
    let inputEl = document.getElementById('user-by-name-input')
    let res = await fetch(`https://localhost:3030/users/get_by_name/${inputEl.textContent}/json`)
    let json = await res.text()
    document.getElementById('user-by-name-output')
        .innerText = json
}