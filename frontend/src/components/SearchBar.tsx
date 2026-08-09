import { useState } from "react";
import '../styles/SearchBar.css';

export default function SearchBar() {
    const [query, setQuery] = useState("");
    return (
        <>
        <section>
            <h1>أين</h1>
            <div>
                <input
                    type="text"
                    placeholder="Ask anything, find everywhere"
                    value={query}
                    onChange={event => setQuery(event.target.value)}
                />
            </div>
        </section>
        </>
    );
}