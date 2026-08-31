Implement the AI layer as a separate, replaceable module.

Potential features:

- AI tutor
- Course Q&A
- Content summarization
- Study recommendations
- Question generation
- Personalized learning recommendations
- At-risk student detection

For course-specific Q&A, use retrieval-augmented generation.

Architecture:

Course Content
    ↓
Document Processing
    ↓
Chunking
    ↓
Embeddings
    ↓
Vector Store
    ↓
Retriever
    ↓
LLM
    ↓
Student

AI responses must clearly distinguish generated answers from official course material.

Do not allow the AI system to expose private course content to unauthorized users.

Record appropriate AI usage metadata without unnecessarily storing sensitive student conversations.